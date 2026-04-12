# agent-harness

A CLI tool that auto-dispatches [conflict-free](https://github.com/thesurlydev/conflict-free) factory tasks to the Claude API and applies the resulting AST-CRDT intents.

## How it works

### Code mode (default)

1. Connects to a running conflict-free orchestrator
2. Polls for tasks with `status=ready`
3. Claims each ready task
4. Fetches current source via `/a2a/source`
5. Sends task description + source context to Claude API
6. Parses the intent JSON array from Claude's response
7. Applies each intent to the orchestrator via `/a2a/intents`
8. Marks the task complete
9. Repeats until no tasks remain
10. Optionally runs a compile-fix loop (`--project-dir`): syncs source to disk, runs `cargo check`, sends errors back to Claude for automated fixes

### PRD mode (`--mode prd`)

1. Selects an agent pool based on `--app-type` (api-only, full-stack, cli)
2. Round 1 -- Parallel Enrichment: all agents generate PRD intents concurrently
3. Round 2 -- Adversarial Review: adversary challenges assumptions, simplifier cuts scope, product manager resolves conflicts
4. Round 3 -- Contract Lock: contract enforcer ensures completeness
5. Runs completeness check and three-layer verification (structural, behavioral, contract)
6. Derives artifacts (OpenAPI spec, test skeletons, verification checklist) and writes them to disk when `--project-dir` is set

## Usage

```bash
export ANTHROPIC_API_KEY=sk-...

agent-harness <ORCHESTRATOR_URL> <TOKEN> <MODEL> <AGENT_NAME> [OPTIONS]
```

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--mode` | `code` | Agent mode: `code` or `prd` |
| `--project-dir` | none | Project directory for compile-fix loop (code mode) or artifact output (PRD mode) |
| `--max-fix-rounds` | `3` | Max rounds for the compile-fix loop |
| `--app-type` | `api-only` | PRD agent pool: `api-only`, `full-stack`, `cli` |

### Examples

```bash
# Start the orchestrator (in conflict-free project directory)
ast-crdt serve --document project.crdt --port 8080 --auth-token mytoken --db orchestrator.redb

# Submit a task DAG
curl -X POST http://localhost:8080/a2a/tasks/batch \
  -H "Authorization: Bearer mytoken" \
  -H "Content-Type: application/json" \
  -d @tasks.json

# Run code generation with compile-fix loop
agent-harness http://localhost:8080 mytoken claude-sonnet-4-6-20250514 builder-1 \
  --project-dir ./my-project

# Run PRD enrichment
agent-harness http://localhost:8080 mytoken claude-sonnet-4-6-20250514 prd-enricher \
  --mode prd --app-type full-stack --project-dir ./my-project
```

## Architecture

```
agent-harness
  src/
    main.rs                 -- CLI entry point (clap), config setup, mode dispatch
    types.rs                -- AgentConfig, AgentMode, Task, IntentWrapper, PRD types
    orchestrator_client.rs  -- HTTP client for conflict-free /a2a/* endpoints
    claude_client.rs        -- HTTP client for Anthropic Messages API
    runner.rs               -- Code mode: poll -> claim -> generate -> apply -> complete + compile-fix loop
    prd_runner.rs           -- PRD mode: 3-round enrichment, verification, artifact derivation
  prompts/
    generate_intents.md     -- Code intent generation prompt (embedded at compile time)
    generate_fixes.md       -- Compile-fix prompt (embedded at compile time)
    prd/                    -- 11 role-specific PRD enrichment prompts (loaded at runtime)
```

## Origin

The initial version was generated in 3 minutes by the [playbook-factory](https://github.com/thesurlydev/playbook-factory) pipeline (run 9) using [conflict-free](https://github.com/thesurlydev/conflict-free) AST-CRDT intents. 31 intents were applied via HTTP with 0 failures and 0 manual fixups, compiled on first try. The codebase has since grown to ~1000 lines across 6 modules with the addition of PRD enrichment, verification, and artifact derivation.

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
