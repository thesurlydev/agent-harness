use crate::types::{AgentConfig, Task, IntentWrapper, PrdIntentWrapper, PrdCompletenessResponse, PrdVerifyResponse, PrdArtifactsResponse};
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
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/a2a/tasks/{}", self.base_url, task_id);
        let body = serde_json::json!(
            { "state" : "working", "assigned_agent" : agent_name }
        );
        let resp = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&body)
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("claim_task {task_id}: HTTP {status} — {body}").into());
        }
        Ok(())
    }
    pub async fn complete_task(&self, task_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/a2a/tasks/{}", self.base_url, task_id);
        let body = serde_json::json!({ "state" : "completed" });
        let resp = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&body)
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("complete_task {task_id}: HTTP {status} — {body}").into());
        }
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
    pub async fn get_prd_source(&self) -> Result<String, reqwest::Error> {
        let url = format!("{}/a2a/prd-source", self.base_url);
        let resp = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send().await?
            .text().await?;
        Ok(resp)
    }
    pub async fn apply_prd_intent(&self, wrapper: &PrdIntentWrapper) -> Result<bool, reqwest::Error> {
        let url = format!("{}/a2a/prd-intents", self.base_url);
        let resp = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&wrapper)
            .send().await?;
        Ok(resp.status().is_success())
    }
    pub async fn get_prd_completeness(&self) -> Result<PrdCompletenessResponse, reqwest::Error> {
        let url = format!("{}/a2a/prd-completeness", self.base_url);
        let resp = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send().await?
            .json::<PrdCompletenessResponse>().await?;
        Ok(resp)
    }

    pub async fn get_prd_artifacts(&self) -> Result<PrdArtifactsResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/a2a/prd-artifacts", self.base_url);
        let resp = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("get_prd_artifacts: HTTP {status} — {body}").into());
        }
        let parsed = resp.json::<PrdArtifactsResponse>().await?;
        Ok(parsed)
    }

    pub async fn verify_prd(&self) -> Result<PrdVerifyResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/a2a/prd-verify", self.base_url);
        let resp = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("verify_prd: HTTP {status} — {body}").into());
        }
        let parsed = resp.json::<PrdVerifyResponse>().await?;
        Ok(parsed)
    }
}


