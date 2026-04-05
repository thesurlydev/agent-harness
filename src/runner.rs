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
}


