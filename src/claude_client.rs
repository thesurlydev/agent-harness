use crate::types::{AgentConfig, ClaudeRequest, ClaudeMessage, ClaudeResponse, PrdAgentRole};

const INTENTS_PROMPT: &str = include_str!("../prompts/generate_intents.md");
const FIXES_PROMPT: &str = include_str!("../prompts/generate_fixes.md");

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
        let prompt = INTENTS_PROMPT
            .replace("<<SOURCE>>", source)
            .replace("<<TASK_DESCRIPTION>>", task_description);
        let req = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 8192,
            messages: vec![ClaudeMessage { role: "user".to_string(), content: prompt }],
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

    pub async fn generate_fixes(
        &self,
        source: &str,
        errors: &str,
    ) -> Result<String, reqwest::Error> {
        let prompt = FIXES_PROMPT
            .replace("<<SOURCE>>", source)
            .replace("<<ERRORS>>", errors);
        let req = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 8192,
            messages: vec![ClaudeMessage { role: "user".to_string(), content: prompt }],
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

    pub async fn generate_prd_intents(
        &self,
        prd_source: &str,
        task_description: &str,
        role_prompt: &str,
    ) -> Result<String, reqwest::Error> {
        let prompt = role_prompt
            .replace("<<PRD_SOURCE>>", prd_source)
            .replace("<<TASK_DESCRIPTION>>", task_description);
        let req = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 8192,
            messages: vec![ClaudeMessage { role: "user".to_string(), content: prompt }],
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

pub fn load_prd_prompt(role: PrdAgentRole) -> String {
    let path = format!("prompts/prd/{}.md", role.as_str());
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to load prompt file {}: {}", path, e))
}
