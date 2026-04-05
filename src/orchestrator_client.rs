use crate::types::{AgentConfig, Task, IntentWrapper};
pub struct OrchestratorClient {
    pub client: reqwest::Client,
    pub base_url: String,
    pub token: String,
}
impl OrchestratorClient {
    pub fn new(config: &AgentConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: config.orchestrator_url.trim_end_matches('/').to_string(),
            token: config.orchestrator_token.clone(),
        }
    }
    pub async fn get_ready_tasks(&self) -> Result<Vec<Task>, reqwest::Error> {
        let url = format!("{}/a2a/tasks?status=ready", self.base_url);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?
            .json::<Vec<Task>>()
            .await?;
        Ok(resp)
    }
    pub async fn claim_task(
        &self,
        task_id: &str,
        agent_name: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/a2a/tasks/{}", self.base_url, task_id);
        let body = serde_json::json!(
            { "state" : "working", "assigned_agent" : agent_name }
        );
        self.client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&body)
            .send()
            .await?;
        Ok(())
    }
    pub async fn complete_task(&self, task_id: &str) -> Result<(), reqwest::Error> {
        let url = format!("{}/a2a/tasks/{}", self.base_url, task_id);
        let body = serde_json::json!({ "state" : "completed" });
        self.client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&body)
            .send()
            .await?;
        Ok(())
    }
    pub async fn get_source(&self) -> Result<String, reqwest::Error> {
        let url = format!("{}/a2a/source", self.base_url);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?
            .text()
            .await?;
        Ok(resp)
    }
    pub async fn apply_intent(
        &self,
        wrapper: &IntentWrapper,
    ) -> Result<bool, reqwest::Error> {
        let url = format!("{}/a2a/intents", self.base_url);
        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&wrapper)
            .send()
            .await?;
        Ok(resp.status().is_success())
    }
}


