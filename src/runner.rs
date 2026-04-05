use crate::types::{AgentConfig, IntentWrapper};
use crate::orchestrator_client::OrchestratorClient;
use crate::claude_client::ClaudeClient;
pub async fn run(config: AgentConfig) {
    let orch = OrchestratorClient::new(&config);
    let claude = ClaudeClient::new(&config);
    println!("Agent {} connecting to {}", config.agent_name, config.orchestrator_url);
    loop {
        let tasks = orch.get_ready_tasks().await.unwrap();
        if tasks.is_empty() {
            println!("No ready tasks. Waiting 5s...");
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            continue;
        }
        for task in &tasks {
            println!("Claiming task: {} - {}", task.id, task.description);
            orch.claim_task(&task.id, &config.agent_name).await.unwrap();
            let source = orch.get_source().await.unwrap();
            println!("Generating intents for task: {}", task.id);
            let response = claude
                .generate_intents(&source, &task.description)
                .await
                .unwrap();
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
                        "Failed to parse intents: {}. Response: {}", e, &
                        cleaned[..cleaned.len().min(500)]
                    );
                    continue;
                }
            };
            let mut pass = 0u32;
            let mut fail = 0u32;
            for wrapper in &intents {
                if orch.apply_intent(wrapper).await.unwrap() {
                    pass += 1;
                } else {
                    fail += 1;
                }
            }
            println!(
                "Task {}: {} intents applied ({} pass, {} fail)", task.id, intents.len(),
                pass, fail
            );
            orch.complete_task(&task.id).await.unwrap();
        }
        let remaining = orch.get_ready_tasks().await.unwrap();
        if remaining.is_empty() {
            println!("All tasks complete.");
            break;
        }
    }

    // Compile-fix loop: sync source, cargo check, fix errors via Claude
    if let Some(ref project_dir) = config.project_dir {
        println!("\n--- Compile-fix loop ---");

        // Sync source from server
        let source = orch.get_source().await.unwrap();
        write_source_files(&source, project_dir);

        for round in 1..=config.max_fix_rounds {
            println!("Fix round {}/{}: running cargo check...", round, config.max_fix_rounds);
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
            println!("cargo check failed: {} errors", error_count);

            // Send errors to Claude for fixes
            let current_source = orch.get_source().await.unwrap();
            let response = claude.generate_fixes(&current_source, &stderr).await.unwrap();

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
                    eprintln!("Failed to parse fix intents: {}", e);
                    continue;
                }
            };

            let mut pass = 0u32;
            let mut fail = 0u32;
            for wrapper in &intents {
                if orch.apply_intent(wrapper).await.unwrap() {
                    pass += 1;
                } else {
                    fail += 1;
                }
            }
            println!("Fix round {}: {} intents ({} pass, {} fail)", round, intents.len(), pass, fail);

            // Re-sync source
            let source = orch.get_source().await.unwrap();
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
            eprintln!("Still {} errors after {} fix rounds.", error_count, config.max_fix_rounds);
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
    println!("Synced {} files to {}", count, project_dir);
}


