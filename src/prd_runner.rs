use crate::claude_client::{load_prd_prompt, ClaudeClient};
use crate::orchestrator_client::OrchestratorClient;
use crate::types::{AgentConfig, PrdAgentRole, PrdIntentWrapper};

pub async fn run_prd_enrichment(config: &AgentConfig) {
    let pool = PrdAgentRole::pool_for_app_type(&config.app_type);
    println!(
        "PRD enrichment: {} agents for app_type={}",
        pool.len(),
        config.app_type
    );

    // Round 1 - Parallel Enrichment
    println!("\n--- Round 1: Parallel Enrichment ---");
    let orch = OrchestratorClient::new(config);
    let prd_source = orch.get_prd_source().await.unwrap();

    let mut handles = Vec::new();
    for &role in &pool {
        let task_desc = task_description_for(role).to_string();
        let prompt = load_prd_prompt(role);
        let prd = prd_source.clone();
        let api_key = config.anthropic_api_key.clone();
        let model = config.model.clone();
        let orch_url = config.orchestrator_url.clone();
        let orch_token = config.orchestrator_token.clone();
        let agent_name = role.as_str().to_string();

        handles.push(tokio::spawn(async move {
            let claude = ClaudeClient {
                client: reqwest::Client::new(),
                api_key,
                model,
            };
            let orch = OrchestratorClient {
                client: reqwest::Client::new(),
                base_url: orch_url.trim_end_matches('/').to_string(),
                token: orch_token,
            };

            println!("[{}] generating intents...", agent_name);
            let response = claude
                .generate_prd_intents(&prd, &task_desc, &prompt)
                .await
                .unwrap();
            let intents = parse_prd_intents(&response, &agent_name);
            apply_intents(&orch, &intents, &agent_name).await;
            (agent_name, intents.len())
        }));
    }

    for handle in handles {
        let (name, count) = handle.await.unwrap();
        println!("[{}] {} intents applied", name, count);
    }

    // Round 2 - Adversarial Review
    println!("\n--- Round 2: Adversarial Review ---");
    let orch = OrchestratorClient::new(config);
    let claude = ClaudeClient::new(config);

    if pool.contains(&PrdAgentRole::Adversary) {
        let prd_source = orch.get_prd_source().await.unwrap();
        let prompt = load_prd_prompt(PrdAgentRole::Adversary);
        let task_desc = "Challenge every assumption in the PRD. Add edge cases and failure modes. Identify missing error handling. Question whether each feature is necessary.";
        println!("[adversary] challenging PRD...");
        let response = claude
            .generate_prd_intents(&prd_source, task_desc, &prompt)
            .await
            .unwrap();
        let intents = parse_prd_intents(&response, "adversary");
        apply_intents(&orch, &intents, "adversary").await;
    }

    if pool.contains(&PrdAgentRole::Simplifier) {
        let prd_source = orch.get_prd_source().await.unwrap();
        let prompt = load_prd_prompt(PrdAgentRole::Simplifier);
        let task_desc = "Apply YAGNI ruthlessly. Remove any feature that lacks a concrete user scenario. Merge redundant endpoints. Simplify the data model.";
        println!("[simplifier] simplifying PRD...");
        let response = claude
            .generate_prd_intents(&prd_source, task_desc, &prompt)
            .await
            .unwrap();
        let intents = parse_prd_intents(&response, "simplifier");
        apply_intents(&orch, &intents, "simplifier").await;
    }

    if pool.contains(&PrdAgentRole::ProductManager) {
        let prd_source = orch.get_prd_source().await.unwrap();
        let prompt = load_prd_prompt(PrdAgentRole::ProductManager);
        let task_desc = "Resolve any conflicts introduced by the adversary and simplifier. Make final priority calls. Ensure every remaining feature has success criteria.";
        println!("[product_manager] resolving conflicts...");
        let response = claude
            .generate_prd_intents(&prd_source, task_desc, &prompt)
            .await
            .unwrap();
        let intents = parse_prd_intents(&response, "product_manager");
        apply_intents(&orch, &intents, "product_manager").await;
    }

    // Round 3 - Contract Lock
    println!("\n--- Round 3: Contract Lock ---");
    if pool.contains(&PrdAgentRole::ContractEnforcer) {
        let prd_source = orch.get_prd_source().await.unwrap();
        let prompt = load_prd_prompt(PrdAgentRole::ContractEnforcer);
        let task_desc = "Final completeness pass. Ensure every route has request/response examples. Add shared type definitions. Verify all sections are filled.";
        println!("[contract_enforcer] locking contract...");
        let response = claude
            .generate_prd_intents(&prd_source, task_desc, &prompt)
            .await
            .unwrap();
        let intents = parse_prd_intents(&response, "contract_enforcer");
        apply_intents(&orch, &intents, "contract_enforcer").await;
    }

    // Completeness report
    let completeness = orch.get_prd_completeness().await.unwrap();
    println!("\n--- PRD Completeness Report ---");
    println!("Complete: {}", completeness.complete);
    println!(
        "Routes: {}/{} with examples, {}/{} with error cases",
        completeness.routes_with_examples,
        completeness.total_routes,
        completeness.routes_with_error_cases,
        completeness.total_routes
    );
    if !completeness.incomplete_sections.is_empty() {
        println!("Incomplete sections: {:?}", completeness.incomplete_sections);
    }
    if !completeness.blocking_flags.is_empty() {
        println!("Blocking flags: {:?}", completeness.blocking_flags);
    }

    // Print final PRD
    let final_prd = orch.get_prd_source().await.unwrap();
    println!("\n--- Final Enriched PRD ---");
    println!("{}", final_prd);
}

fn parse_prd_intents(response: &str, agent_name: &str) -> Vec<PrdIntentWrapper> {
    let cleaned = response.trim();
    let cleaned = if cleaned.starts_with("```") {
        let start = cleaned.find('\n').map(|i| i + 1).unwrap_or(0);
        let end = cleaned[start..].rfind("```").map(|i| start + i).unwrap_or(cleaned.len());
        if end > start { &cleaned[start..end] } else { &cleaned[start..] }
    } else {
        cleaned
    };
    match serde_json::from_str::<Vec<PrdIntentWrapper>>(cleaned) {
        Ok(intents) => {
            println!("[{}] parsed {} intents", agent_name, intents.len());
            intents
        }
        Err(e) => {
            eprintln!(
                "[{}] failed to parse intents: {}. Response: {}",
                agent_name,
                e,
                &cleaned[..cleaned.len().min(500)]
            );
            Vec::new()
        }
    }
}

async fn apply_intents(orch: &OrchestratorClient, intents: &[PrdIntentWrapper], agent_name: &str) {
    let mut pass = 0u32;
    let mut fail = 0u32;
    for wrapper in intents {
        match orch.apply_prd_intent(wrapper).await {
            Ok(true) => pass += 1,
            Ok(false) => fail += 1,
            Err(e) => {
                eprintln!("[{}] intent apply error: {}", agent_name, e);
                fail += 1;
            }
        }
    }
    println!(
        "[{}] applied: {} pass, {} fail",
        agent_name, pass, fail
    );
}

fn task_description_for(role: PrdAgentRole) -> &'static str {
    match role {
        PrdAgentRole::ProductManager => "Review the draft PRD. Set success metrics for every feature. Define non-goals. Establish priority ordering.",
        PrdAgentRole::ApiArchitect => "Review the draft PRD. Add example request/response pairs for every route. Ensure RESTful conventions.",
        PrdAgentRole::RustEngineer => "Review the draft PRD. Define error handling strategy. Add newtypes. Set crate choices.",
        PrdAgentRole::Dba => "Review the draft PRD. Design PostgreSQL schema with tables, indexes, foreign keys, migration ordering.",
        PrdAgentRole::SecurityAuditor => "Review the draft PRD. Add validation rules for every input. Set auth requirements. Configure rate limits.",
        PrdAgentRole::Adversary => "Review the draft PRD. Add edge cases and error cases. Challenge assumptions.",
        PrdAgentRole::Simplifier => "Review the draft PRD. Remove features without user scenarios. Merge redundant endpoints.",
        PrdAgentRole::ContractEnforcer => "Review the draft PRD. Ensure every route has examples. Add shared type definitions.",
        PrdAgentRole::OpsEngineer => "Review the draft PRD. Add health checks. Define env vars. Set logging and pool config.",
        PrdAgentRole::FrontendArchitect => "Review the draft PRD. Define pages, components, data requirements, loading states.",
        PrdAgentRole::DesignLead => "Review the draft PRD. Define design tokens, layout specs, responsive rules, accessibility.",
    }
}
