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
            "You are implementing a module for a Rust project. Output ast-crdt intents as JSON.\n\n## Current Source\n{}\n\n## Task\n{}\n\n## Rules\n- Each intent is wrapped as: {{\"intent\": {{...}}}}\n- Use the correct intent format with \"type\" field\n- Output a JSON array of wrapped intents, nothing else\n- See the factory playbook for the full intent reference",
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


