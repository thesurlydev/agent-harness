use crate::types::{AgentConfig, ClaudeRequest, ClaudeMessage, ClaudeResponse};
pub struct ClaudeClient {
    pub client: reqwest::Client,
    pub api_key: String,
    pub model: String,
}
impl ClaudeClient {
    pub fn new(config: &AgentConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: config.anthropic_api_key.clone(),
            model: config.model.clone(),
        }
    }
    pub async fn generate_intents(
        &self,
        source: &str,
        task_description: &str,
    ) -> Result<String, reqwest::Error> {
        let prompt = format!(
            "You are implementing a module for a Rust project using ast-crdt intents.\n\n## Current Source\n{}\n\n## Task\n{}\n\n## Available Intent Types\n- add_use: {{\"type\":\"add_use\",\"path\":\"...\",\"file\":\"...\"}}\n- add_struct: {{\"type\":\"add_struct\",\"name\":\"...\",\"visibility\":\"pub\",\"attributes\":[\"#[derive(...)]\"],\"fields\":[{{\"name\":\"...\",\"type\":\"...\",\"visibility\":\"pub\"}}],\"file\":\"...\"}}\n- add_enum: {{\"type\":\"add_enum\",\"name\":\"...\",\"visibility\":\"pub\",\"variants\":[\"A\",\"B\"],\"attributes\":[],\"file\":\"...\"}}\n- add_function: {{\"type\":\"add_function\",\"name\":\"...\",\"visibility\":\"pub\",\"is_async\":false,\"params\":[[\"arg\",\"Type\"]],\"returns\":\"RetType\",\"body\":\"{{ ... }}\",\"file\":\"...\"}}\n- add_impl: {{\"type\":\"add_impl\",\"target_type\":\"...\",\"trait_name\":null,\"file\":\"...\"}}\n- add_method: {{\"type\":\"add_method\",\"impl_target\":\"...\",\"impl_trait\":null,\"name\":\"...\",\"visibility\":\"pub\",\"receiver\":\"&self\",\"returns\":\"...\",\"body\":\"{{ ... }}\",\"is_async\":false,\"file\":\"...\"}}\n- set_function_body: {{\"type\":\"set_function_body\",\"fn_name\":\"...\",\"body\":\"{{ ... }}\",\"file\":\"...\"}}\n- add_attribute: {{\"type\":\"add_attribute\",\"target\":\"...\",\"kind\":\"function\",\"attribute\":\"#[tokio::main]\",\"file\":\"...\"}}\n\n## Rules\n- Output ONLY a JSON array, no markdown, no explanation\n- Each element is: {{\"intent\": {{<intent object>}}}}\n- Function bodies must be valid Rust wrapped in {{ }}\n- Use tokio::sync::Mutex (not std::sync::Mutex) for Axum shared state\n- Use {{param}} route syntax for Axum 0.8",
            source, task_description
        );
        let req = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 8192,
            messages: vec![ClaudeMessage { role : "user".to_string(), content : prompt }],
        };
        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&req)
            .send()
            .await?
            .json::<ClaudeResponse>()
            .await?;
        let text = resp
            .content
            .into_iter()
            .filter_map(|c| c.text)
            .collect::<Vec<_>>()
            .join("");
        Ok(text)
    }
}


