use clap::Parser;
use crate::types::AgentConfig;
mod claude_client;
mod orchestrator_client;
mod runner;
mod types;
#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let api_key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY not set");
    let config = AgentConfig {
        orchestrator_url: cli.orchestrator_url,
        orchestrator_token: cli.token,
        anthropic_api_key: api_key,
        model: cli.model,
        agent_name: cli.agent_name,
        project_dir: cli.project_dir,
        max_fix_rounds: cli.max_fix_rounds,
    };
    runner::run(config).await;
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
}


