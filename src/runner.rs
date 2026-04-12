use crate::claude_client::ClaudeClient;
use crate::orchestrator_client::OrchestratorClient;
use crate::types::{AgentConfig, IntentWrapper};

pub async fn run(config: AgentConfig) {
    let orch = OrchestratorClient::new(&config);
    let claude = ClaudeClient::new(&config);
    println!(
        "Agent {} connecting to {}",
        config.agent_name, config.orchestrator_url
    );

    let mut consecutive_empties = 0u32;
    loop {
        let tasks = match orch.get_ready_tasks().await {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed to fetch ready tasks: {e}");
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                consecutive_empties += 1;
                if consecutive_empties >= 5 {
                    eprintln!("No reachable tasks after 5 attempts. Exiting.");
                    break;
                }
                continue;
            }
        };

        if tasks.is_empty() {
            consecutive_empties += 1;
            if consecutive_empties >= 3 {
                println!("No ready tasks after {consecutive_empties} polls. DAG complete or stalled.");
                break;
            }
            println!("No ready tasks. Waiting 5s... ({consecutive_empties}/3)");
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            continue;
        }
        consecutive_empties = 0;

        for task in &tasks {
            println!("Claiming task: {} - {}", task.id, task.description);
            if let Err(e) = orch.claim_task(&task.id, &config.agent_name).await {
                eprintln!("  Failed to claim task {}: {e}. Skipping.", task.id);
                continue;
            }

            let source = match orch.get_source().await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("  Failed to get source for task {}: {e}. Skipping.", task.id);
                    continue;
                }
            };

            println!("Generating intents for task: {}", task.id);
            let response = match claude
                .generate_intents(&source, &task.description)
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    eprintln!(
                        "  Claude API failed for task {}: {e}. Marking complete to unblock DAG.",
                        task.id
                    );
                    let _ = orch.complete_task(&task.id, &config.agent_name).await;
                    continue;
                }
            };

            // Strip markdown code fences if present
            let cleaned = response.trim();
            let cleaned = if cleaned.starts_with("```") {
                let start = cleaned.find('\n').map(|i| i + 1).unwrap_or(0);
                let end = cleaned.rfind("```").unwrap_or(cleaned.len());
                &cleaned[start..end]
            } else {
                cleaned
            };
            let intents: Vec<IntentWrapper> = match serde_json::from_str(cleaned) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!(
                        "  Failed to parse intents for task {}: {}. Response: {}",
                        task.id,
                        e,
                        &cleaned[..cleaned.len().min(500)]
                    );
                    let _ = orch.complete_task(&task.id, &config.agent_name).await;
                    continue;
                }
            };

            let mut pass = 0u32;
            let mut fail = 0u32;
            for wrapper in &intents {
                match orch.apply_intent(wrapper).await {
                    Ok(true) => pass += 1,
                    Ok(false) => fail += 1,
                    Err(e) => {
                        eprintln!("  Intent apply error: {e}");
                        fail += 1;
                    }
                }
            }
            println!(
                "Task {}: {} intents applied ({} pass, {} fail)",
                task.id,
                intents.len(),
                pass,
                fail
            );

            // Try to complete the task. Retry once on failure (validation
            // gate can reject transiently if a concurrent intent is still
            // being applied).
            if let Err(e) = orch.complete_task(&task.id, &config.agent_name).await {
                eprintln!("  complete_task {} failed: {e}. Retrying in 2s...", task.id);
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                if let Err(e2) = orch.complete_task(&task.id, &config.agent_name).await {
                    eprintln!(
                        "  complete_task {} failed again: {e2}. Task may be stuck in WORKING.",
                        task.id
                    );
                }
            }
        }
    }

    // Compile-fix loop: sync source, cargo check, fix errors via Claude
    if let Some(ref project_dir) = config.project_dir {
        println!("\n--- Compile-fix loop ---");

        let source = match orch.get_source().await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to sync source: {e}");
                return;
            }
        };
        write_source_files(&source, project_dir);

        for round in 1..=config.max_fix_rounds {
            println!(
                "Fix round {}/{}: running cargo check...",
                round, config.max_fix_rounds
            );
            let output = std::process::Command::new("cargo")
                .arg("check")
                .current_dir(project_dir)
                .output()
                .expect("Failed to run cargo check");

            if output.status.success() {
                println!("cargo check passed! Build successful.");
                return;
            }

            let stderr = String::from_utf8_lossy(&output.stderr);
            let error_count = stderr.matches("error[E").count();
            println!("cargo check failed: {error_count} errors");

            let current_source = match orch.get_source().await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to sync source for fix round: {e}");
                    continue;
                }
            };
            let response = match claude.generate_fixes(&current_source, &stderr).await {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Claude API failed for fix round {round}: {e}");
                    continue;
                }
            };

            let cleaned = response.trim();
            let cleaned = if cleaned.starts_with("```") {
                let start = cleaned.find('\n').map(|i| i + 1).unwrap_or(0);
                let end = cleaned.rfind("```").unwrap_or(cleaned.len());
                &cleaned[start..end]
            } else {
                cleaned
            };

            let intents: Vec<IntentWrapper> = match serde_json::from_str(cleaned) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Failed to parse fix intents: {e}");
                    continue;
                }
            };

            let mut pass = 0u32;
            let mut fail = 0u32;
            for wrapper in &intents {
                match orch.apply_intent(wrapper).await {
                    Ok(true) => pass += 1,
                    Ok(false) => fail += 1,
                    Err(e) => {
                        eprintln!("  Fix intent apply error: {e}");
                        fail += 1;
                    }
                }
            }
            println!(
                "Fix round {round}: {} intents ({pass} pass, {fail} fail)",
                intents.len()
            );

            let source = match orch.get_source().await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to re-sync source: {e}");
                    continue;
                }
            };
            write_source_files(&source, project_dir);
        }

        // Final check
        let output = std::process::Command::new("cargo")
            .arg("check")
            .current_dir(project_dir)
            .output()
            .expect("Failed to run cargo check");
        if output.status.success() {
            println!("cargo check passed after fix rounds!");
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let error_count = stderr.matches("error[E").count();
            eprintln!(
                "Still {error_count} errors after {} fix rounds.",
                config.max_fix_rounds
            );
        }
    }
}

fn write_source_files(source: &str, project_dir: &str) {
    let mut current_path: Option<String> = None;
    let mut current_lines: Vec<&str> = Vec::new();
    let mut count = 0u32;

    for line in source.lines() {
        if line.starts_with("// === ") && line.ends_with(" ===") {
            if let Some(ref path) = current_path {
                let dest = std::path::Path::new(project_dir).join(path);
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent).ok();
                }
                std::fs::write(&dest, current_lines.join("\n") + "\n").ok();
                count += 1;
            }
            current_path = Some(line[7..line.len() - 4].to_string());
            current_lines.clear();
        } else {
            current_lines.push(line);
        }
    }
    if let Some(ref path) = current_path {
        let dest = std::path::Path::new(project_dir).join(path);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&dest, current_lines.join("\n") + "\n").ok();
        count += 1;
    }
    println!("Synced {count} files to {project_dir}");
}
