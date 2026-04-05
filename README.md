# agent-harness

A CLI tool that auto-dispatches [conflict-free](https://github.com/thesurlydev/conflict-free) factory tasks to the Claude API and applies the resulting AST-CRDT intents.

## How it works

1. Connects to a running conflict-free orchestrator
2. Polls for tasks with `status=ready`
3. Claims each ready task
4. Fetches current source via `/a2a/source`
5. Sends task description + source context to Claude API
6. Parses the intent JSON array from Claude's response
7. Applies each intent to the orchestrator via `/a2a/intents`
8. Marks the task complete
9. Repeats until no tasks remain

## Usage

```bash
export ANTHROPIC_API_KEY=sk-...

agent-harness <ORCHESTRATOR_URL> <TOKEN> <MODEL> <AGENT_NAME>
```

Example:

```bash
# Start the orchestrator (in conflict-free project directory)
ast-crdt serve --document project.crdt --port 8080 --auth-token mytoken --db orchestrator.redb

# Submit a task DAG
curl -X POST http://localhost:8080/a2a/tasks/batch \
  -H "Authorization: Bearer mytoken" \
  -H "Content-Type: application/json" \
  -d @tasks.json

# Run the agent harness
agent-harness http://localhost:8080 mytoken claude-sonnet-4-6-20250514 builder-1
```

The harness will claim ready tasks in dependency order, generate intents via Claude, apply them, and move on until the DAG is fully resolved.

## Architecture

```
agent-harness
  src/
    main.rs                 -- CLI entry point (clap), config setup
    types.rs                -- AgentConfig, Task, IntentWrapper, Claude API types
    orchestrator_client.rs  -- HTTP client for conflict-free orchestrator
    claude_client.rs        -- HTTP client for Anthropic Messages API
    runner.rs               -- Main loop: poll -> claim -> generate -> apply -> complete
```

## Origin

This project was generated in 3 minutes by the [playbook-factory](https://github.com/thesurlydev/playbook-factory) pipeline (run 9) using [conflict-free](https://github.com/thesurlydev/conflict-free) AST-CRDT intents. 31 intents were applied via HTTP with 0 failures and 0 manual fixups. The entire codebase (260 lines across 5 modules) was produced by the factory's intent application system, synced via `--from-server`, and compiled on first try.

This is the first factory product that is itself a component of the factory pipeline -- it automates the "human runs curl commands" step that was manual in runs 1-8.

## Requirements

- Rust 2024 edition
- A running conflict-free orchestrator with a task DAG
- `ANTHROPIC_API_KEY` environment variable set
- Network access to both the orchestrator and `api.anthropic.com`

## Building

```bash
cargo build --release
```

## Related projects

- [conflict-free](https://github.com/thesurlydev/conflict-free) -- AST-CRDT orchestrator for multi-agent code generation
- [playbook-factory](https://github.com/thesurlydev/playbook-factory) -- Factory playbook and run artifacts
- [fifty](https://github.com/thesurlydev/fifty) -- Zero-dependency deployment tool
