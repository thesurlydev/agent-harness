use serde::{Serialize, Deserialize};
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub orchestrator_url: String,
    pub orchestrator_token: String,
    pub anthropic_api_key: String,
    pub model: String,
    pub agent_name: String,
    pub project_dir: Option<String>,
    pub max_fix_rounds: u32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub state: String,
    pub target_path: Option<String>,
    pub depends_on: Option<Vec<String>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentWrapper {
    pub intent: serde_json::Value,
}
#[derive(Debug, Serialize)]
pub struct ClaudeMessage {
    pub role: String,
    pub content: String,
}
#[derive(Debug, Serialize)]
pub struct ClaudeRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<ClaudeMessage>,
}
#[derive(Debug, Deserialize)]
pub struct ClaudeResponse {
    pub content: Vec<ClaudeContent>,
}
#[derive(Debug, Deserialize)]
pub struct ClaudeContent {
    pub text: Option<String>,
}
