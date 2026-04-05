use clap::Parser;
use crate::types::{AgentConfig, AgentMode};
mod claude_client;
mod orchestrator_client;
mod prd_runner;
mod runner;
mod types;
#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let api_key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY not set");
    let mode = match cli.mode.as_str() {
        "prd" => AgentMode::Prd,
        _ => AgentMode::Code,
    };
    let config = AgentConfig {
        orchestrator_url: cli.orchestrator_url,
        orchestrator_token: cli.token,
        anthropic_api_key: api_key,
        model: cli.model,
        agent_name: cli.agent_name,
        project_dir: cli.project_dir,
        max_fix_rounds: cli.max_fix_rounds,
        mode,
        app_type: cli.app_type,
    };
    match config.mode {
        AgentMode::Code => runner::run(config).await,
        AgentMode::Prd => prd_runner::run_prd_enrichment(&config).await,
    }
}
#[derive(Parser)]
#[command(name = "agent-harness", about = "Auto-dispatch factory tasks to Claude API")]
struct Cli {
    pub orchestrator_url: String,
    pub token: String,
    pub model: String,
    pub agent_name: String,
    /// Project directory for cargo check fix loop (enables compile-fix cycle after all tasks)
    #[arg(long)]
    pub project_dir: Option<String>,
    /// Max fix rounds for compile-fix loop
    #[arg(long, default_value = "3")]
    pub max_fix_rounds: u32,
    /// Agent mode: "code" or "prd"
    #[arg(long, default_value = "code")]
    pub mode: String,
    /// Application type: "api-only", "full-stack", "cli"
    #[arg(long, default_value = "api-only")]
    pub app_type: String,
}


