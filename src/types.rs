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
    pub mode: AgentMode,
    pub app_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrdIntentWrapper {
    pub prd_intent: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentMode {
    Code,
    Prd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrdAgentRole {
    ProductManager,
    ApiArchitect,
    RustEngineer,
    Dba,
    FrontendArchitect,
    DesignLead,
    ContractEnforcer,
    SecurityAuditor,
    Adversary,
    Simplifier,
    OpsEngineer,
}

impl PrdAgentRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ProductManager => "product_manager",
            Self::ApiArchitect => "api_architect",
            Self::RustEngineer => "rust_engineer",
            Self::Dba => "dba",
            Self::FrontendArchitect => "frontend_architect",
            Self::DesignLead => "design_lead",
            Self::ContractEnforcer => "contract_enforcer",
            Self::SecurityAuditor => "security_auditor",
            Self::Adversary => "adversary",
            Self::Simplifier => "simplifier",
            Self::OpsEngineer => "ops_engineer",
        }
    }

    pub fn pool_for_app_type(app_type: &str) -> Vec<PrdAgentRole> {
        match app_type {
            "full-stack" => vec![
                Self::ProductManager, Self::ApiArchitect, Self::RustEngineer,
                Self::Dba, Self::FrontendArchitect, Self::DesignLead,
                Self::ContractEnforcer, Self::SecurityAuditor, Self::Adversary,
                Self::Simplifier, Self::OpsEngineer,
            ],
            "api-only" => vec![
                Self::ProductManager, Self::ApiArchitect, Self::RustEngineer,
                Self::Dba, Self::ContractEnforcer, Self::SecurityAuditor,
                Self::Adversary, Self::Simplifier, Self::OpsEngineer,
            ],
            "cli" => vec![
                Self::ProductManager, Self::RustEngineer, Self::SecurityAuditor,
                Self::Adversary, Self::Simplifier,
            ],
            _ => Self::pool_for_app_type("api-only"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrdCompletenessResponse {
    pub complete: bool,
    pub incomplete_sections: Vec<String>,
    pub blocking_flags: Vec<String>,
    pub total_routes: usize,
    pub routes_with_examples: usize,
    pub routes_with_error_cases: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrdVerifyResponse {
    pub report: VerificationReport,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VerificationReport {
    pub product_name: String,
    pub items: Vec<VerificationItem>,
    pub structural_pass: usize,
    pub structural_fail: usize,
    pub behavioral_pass: usize,
    pub behavioral_fail: usize,
    pub behavioral_skipped: usize,
    pub contract_pass: usize,
    pub contract_fail: usize,
    pub contract_warnings: usize,
    pub total_checks: usize,
    pub total_pass: usize,
    pub total_fail: usize,
    pub score_percent: f64,
    pub blocking_failures: Vec<String>,
    pub deploy_ready: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VerificationItem {
    pub id: String,
    pub layer: String,
    pub check: String,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrdArtifactsResponse {
    pub openapi: serde_json::Value,
    pub test_skeletons: String,
    pub verification_checklist: serde_json::Value,
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
