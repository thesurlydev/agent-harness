# Forge: Architect + Specialists Code Generation Harness

**Date**: 2026-04-05
**Status**: Draft
**Author**: Shane + Claude

## Overview

Forge is a new Rust CLI tool that takes a rough application spec and produces a fully deployed, verified, and scored Rust/Axum/sqlx web application with optional Vite/React frontend. It replaces both agent-harness and hybrid-forge with a purpose-built pipeline optimized for code quality, correctness, speed/autonomy, and observability.

### Key decisions

- **Direct file generation** -- no conflict-free/AST-CRDT dependency. Claude produces complete files.
- **Architect + Specialists model** -- one Claude call produces a detailed blueprint (the shared contract), then specialist calls generate each file with the blueprint as context.
- **Single CLI with checkpoints** -- one command runs everything; phase artifacts written to disk enable resume-on-failure and full observability.
- **Target stack**: Rust/Axum/sqlx with optional Vite/React frontend, deployed via fifty.

### Priorities

1. **Code quality** -- blueprint enforces consistent architecture before any code is written
2. **Correctness** -- focused specialist calls, iterative compiler loop, generated tests with coverage scoring
3. **Speed/autonomy** -- parallel generation within rounds, surgical fix loops, fully hands-off from spec to deployed
4. **Observability** -- every phase boundary is a checkpoint with structured telemetry, culminating in a quality score

## Project Structure

```
forge/
├── Cargo.toml              # workspace root
├── crates/
│   ├── forge-cli/          # CLI entry point (clap)
│   ├── forge-core/         # Pipeline orchestration, phase execution
│   ├── forge-claude/       # Claude API client (Messages API)
│   ├── forge-compiler/     # Cargo check/test runner, error parsing
│   ├── forge-deployer/     # Fifty CLI driver (shell-out)
│   ├── forge-verifier/     # Post-deploy smoke tests, health checks
│   └── forge-telemetry/    # Structured telemetry, quality scoring, reports
├── prompts/                # Prompt templates (embedded at compile time)
│   ├── architect.md        # Spec -> blueprint expansion
│   ├── specialist/
│   │   ├── types.md        # Types/models generation
│   │   ├── handlers.md     # Route handlers
│   │   ├── storage.md      # Database layer (sqlx)
│   │   ├── migrations.md   # SQL migrations
│   │   ├── tests.md        # Test generation
│   │   └── frontend.md     # React/Vite components (when applicable)
│   └── fix.md              # Targeted compilation fix
└── templates/              # Scaffold templates (Cargo.toml, build configs, etc.)
```

### Crate responsibilities

| Crate | Purpose | Key dependencies |
|-------|---------|-----------------|
| `forge-cli` | CLI parsing, checkpoint resume, human-readable output | clap |
| `forge-core` | Phase orchestration, generation ordering, blueprint validation | forge-claude, forge-compiler, forge-deployer, forge-verifier, forge-telemetry |
| `forge-claude` | Claude Messages API client, prompt template rendering | reqwest, serde_json |
| `forge-compiler` | Run cargo check/test, parse JSON diagnostics, group errors by file | tokio (process), serde_json |
| `forge-deployer` | Shell out to fifty CLI, parse output, manage app.toml/.env | tokio (process) |
| `forge-verifier` | HTTP smoke tests, TLS checks, health checks against deployed app | reqwest |
| `forge-telemetry` | API call logging, phase timing, quality scoring, report generation | serde_json, chrono |

## CLI Interface

```bash
# Full run: spec -> deployed + verified
forge build spec.md --deploy-env production

# Resume from last checkpoint
forge build spec.md --resume

# Dry run: generate + test, don't deploy
forge build spec.md --dry-run

# Just expand spec to blueprint (inspect before building)
# Writes blueprint.json to .forge/runs/<run-id>/ and stops.
# Useful for reviewing the architecture before committing to a full build.
forge plan spec.md

# Resume a specific run from a specific phase
forge resume <run-id> --from compile

# View run report
forge report <run-id>
```

## Checkpoint Directory

Written progressively as the run executes:

```
.forge/runs/<run-id>/
├── meta.json               # Run config, timestamps, status
├── spec.md                 # Input spec (copied)
├── blueprint.json          # Phase 1 output: architecture plan
├── files/                  # Phase 2 output: generated source files
├── compile-log.json        # Phase 3: cargo check/test results per round
├── test-results.json       # Phase 4: test generation + execution results
├── deploy-log.json         # Phase 5: fifty output, server state
├── verification.json       # Phase 6: smoke test results
├── report.json             # Final quality report
└── telemetry/
    ├── api-calls.jsonl     # Every Claude API call (request/response/tokens/latency)
    └── phases.jsonl        # Phase-level timing and status
```

## Phase 1: Blueprint (Architect)

### Input

A rough spec from the user -- key decisions made (database choice, auth strategy, main entities), but the harness fills in architecture. Example:

```markdown
# Link Shortener

URL shortening service with click tracking.

- Rust/Axum, PostgreSQL via sqlx
- Endpoints: create short link, redirect, click stats
- Auth: API key header for create/stats, no auth for redirect
- React dashboard showing click stats over time
```

### Output

`blueprint.json` -- a comprehensive, machine-readable plan that every subsequent phase references. Contains the following sections:

**project** -- name, Rust edition (2024), description

**dependencies** -- crate name, version, features. Used to generate Cargo.toml.

**data_model** -- tables with columns, types, constraints. Used by migration specialist and DB specialist. SQL types (not Rust types) since this drives migration generation.

**types** -- Rust struct definitions with field names and types. Every type referenced in routes must appear here. This is the shared vocabulary between handlers, DB layer, and tests.

**routes** -- method, path, handler function name, auth requirement, request type, response type. This is the contract that handler specialists implement and test specialists verify.

**files** -- ordered manifest of every file to generate, with its purpose. No file is generated that isn't in this list.

**infrastructure** -- deployment and operational configuration:

- `deployment` -- fifty environment, server type, location, domains, backend address. Generates app.toml.
- `database` -- engine, version, cluster name, port, database name, username, and the exact `fifty db` commands to set it up.
- `secrets` -- every secret the app needs, with one of three sources:
  - `derived` -- computed from other blueprint values (e.g., DATABASE_URL from db config)
  - `generated` -- harness creates at build time (random API keys, tokens)
  - `user_provided` -- must exist in .env or environment; harness validates before deploy. Can be marked `required: false`.
- `static_root` -- where frontend assets are served from on the VPS
- `frontend_build` -- build command, output directory, deploy target

**frontend** -- tool (vite), framework (react), entry directory, page/component manifest. Only present when the spec includes a frontend.

**verification** -- pre-planned smoke tests with method, path, body, expected status, expected body content. Dynamic values use `<placeholder>` syntax for test chaining.

### Blueprint validation

Before proceeding to Phase 2, the harness validates:

- Every type referenced in routes exists in `types`
- Every table referenced in types has a corresponding migration in `files`
- Every handler in routes has a corresponding file in `files`
- File manifest covers all routes, types, and migrations
- No circular dependencies between files
- Every secret referenced in code patterns has an entry in `secrets`
- Database config matches sqlx DATABASE_URL pattern
- Domains are specified (required for deployment)
- Required user-provided secrets are flagged for pre-deploy validation

## Phase 2: Code Generation (Specialists)

### Generation order

Files are generated in topological rounds based on their dependencies:

```
Round 1 (no dependencies -- deterministic, no Claude calls):
  - Cargo.toml          (rendered from templates/ using blueprint.dependencies)
  - migrations/*.sql    (rendered from templates/ using blueprint.data_model)
  - app.toml            (rendered from templates/ using blueprint.infrastructure)
  - .env                (computed from blueprint.infrastructure.secrets)

Round 2 (depends on blueprint.types only):
  - src/errors.rs
  - src/types.rs

Round 3 (depends on types):
  - src/db.rs
  - src/auth.rs

Round 4 (depends on db + auth + types):
  - src/handlers.rs

Round 5 (depends on handlers + auth):
  - src/main.rs

Round 6 (if frontend -- independent of Rust rounds):
  - frontend/package.json
  - frontend/vite.config.ts
  - frontend/src/App.tsx
  - frontend/src/components/*.tsx

Round 7 (depends on all code):
  - tests/api_tests.rs
```

### Specialist call context

Each specialist Claude call receives:

1. **Full blueprint** -- the contract (types, routes, data model, infrastructure)
2. **Specialist prompt** -- from `prompts/specialist/<name>.md` (role, rules, Axum/sqlx patterns)
3. **Already-generated dependency files** -- files from earlier rounds that this file imports from
4. **Output format** -- return complete file content, not diffs or intents

### Specialist prompt design

- Include Axum 0.8 / sqlx 0.8 / tokio patterns that Claude must get right (State extraction, `{param}` route syntax, sqlx query macros)
- Reference blueprint by section name ("implement the handlers defined in `blueprint.routes`")
- Specify what "good" looks like for each file type: error handling pattern, naming conventions, import style
- Prompts are `.md` files embedded via `include_str!` -- editable during development, baked in for release

### Parallelism

Files within the same round have no dependencies on each other. Their Claude calls run concurrently. Round 1 fires up to 4 parallel calls, Round 2 fires 2, etc.

### Per-file validation

After each file is generated:

- Syntax check: can `rustfmt` parse it?
- Import check: does it reference types/functions from the blueprint or already-generated files?
- If validation fails: one targeted retry with the error before moving to next round

## Phase 3: Compiler Loop

### Structure

```
Round 1: cargo check
  -> Clean? -> Proceed to cargo test
  -> Errors? -> Parse, group by file

For each file with errors:
  -> Claude call: blueprint + file content + errors + dependency files
  -> Receive corrected file
  -> Write to disk

Round 2: cargo check
  -> Repeat until clean or max rounds (5)

Then: cargo test
  -> Failures? -> Same fix loop (max 3 test-fix rounds)
```

### Error parsing

`forge-compiler` parses `cargo check --message-format=json`:

- Groups diagnostics by file path
- Extracts: error code, message, span (file:line:col), rustc-suggested fix
- Distinguishes errors (must fix) from warnings (log, continue)

### Fix call context

Each fix call receives:

1. Blueprint (contract stays constant)
2. Broken file's current content
3. Specific errors for that file only
4. Files that the broken file imports from
5. `prompts/fix.md` template (rules: minimal changes, fix exactly what's broken, don't redesign)

### Circuit breaker

If the same error persists across 2 consecutive rounds for the same file: log as unresolvable, include in report. No infinite loops.

### Telemetry per round

- Error count before and after
- Which files were fixed
- Tokens spent on fix calls
- Time per round

## Phase 4: Test Generation

After the project compiles and any existing tests pass.

### What gets generated

**Integration tests** (`tests/api_tests.rs`):
- One test per route from `blueprint.routes`
- Happy path: expected status code and response shape
- Auth rejection: wrong/missing API key for authenticated routes
- Validation: missing required fields, invalid input
- Uses real test database (`#[sqlx::test]` fixtures, not mocks)

**Database tests** (in `tests/db_tests.rs` or inline `#[cfg(test)]`):
- Each query function tested against real PostgreSQL
- Insert, query, verify round-trip
- Constraint edge cases: unique violations, foreign key enforcement

**Property-based tests** (when applicable):
- Generated identifiers are valid URL segments
- URL round-trip through create + redirect preserves target
- Input validation rejects all invalid inputs

### Test generation call context

- Blueprint (especially `routes` and `data_model`)
- All generated source files
- `prompts/specialist/tests.md` (patterns: `#[sqlx::test]`, assertion style)

### Test validation

- Compile check: `cargo test --no-run`
- Execute: `cargo test`
- Failures go through Phase 3 fix loop (max 3 rounds)
- Unfixable tests are commented out with `// FIXME:` and logged

### Test quality score

- Route coverage: percentage of `blueprint.routes` with at least one test
- Auth coverage: all authenticated routes tested with and without auth
- Error path coverage: validation failures tested
- Target: 100% route coverage, 100% auth coverage, >50% error paths

## Phase 5: Deploy

The harness drives `fifty` via shell commands.

### Pre-deploy checklist

All must pass before any infrastructure is touched:

- `cargo test` passes
- Required user-provided secrets exist in `.env` or environment
- Domains are specified in blueprint
- If frontend: `npm run build` succeeded, dist directory exists

### Deploy sequence

```
Step 1: Generate config files
  - Write app.toml from blueprint.infrastructure.deployment
  - Write .env with derived + generated secrets

Step 2: Provision infrastructure
  - fifty init --vps <environment>
  - Wait for VPS ready

Step 3: Database setup
  - Execute blueprint.infrastructure.database.setup_commands in order
  - Run migrations: connect via sqlx, apply migrations/*.sql

Step 4: Upload frontend (if applicable)
  - fifty static deploy (uploads frontend/dist to static_root)

Step 5: Deploy application
  - fifty deploy --vps <environment>
  - Wait for service to start (poll fifty status)

Step 6: Checkpoint
  - Write deploy-log.json: server IP, service status, timing
```

### Failure handling

- `fifty init` fails: log, stop. Infrastructure provisioning is not retryable blindly.
- Database setup fails: log, stop. Don't deploy an app that can't reach its DB.
- `fifty deploy` fails: log, capture systemd journal output for diagnostics.
- All failures write to `deploy-log.json` with full context.

## Phase 6: Verification + Quality Report

### Smoke tests

Driven from `blueprint.verification.smoke_tests`:

1. Build HTTP request (method, path, headers, body)
2. Substitute dynamic values (`<short_code>`, `<id>`) from earlier test responses
3. Send request to `https://<domain>`
4. Check status code matches `expect_status`
5. Check body contains `expect_body_contains` (if specified)
6. Record: latency, status, pass/fail, response body snippet

Smoke tests run in order. Dynamic values from earlier responses are captured and substituted into later tests (test chaining).

### Health checks

- Database connectivity: `SELECT 1` through health endpoint or `fifty db status`
- TLS validity: certificate subject matches domain, not expired
- Frontend loads: GET `/` or `/dashboard` returns HTML with expected markers

### Quality report

`report.json` structure:

```json
{
  "run_id": "<timestamp>-<project>-<hash>",
  "status": "success | partial | failed",
  "input_spec": "spec.md",
  "timing": {
    "total_seconds": 187,
    "phase_1_blueprint": 8,
    "phase_2_generation": 24,
    "phase_3_compiler_loop": 15,
    "phase_4_test_generation": 18,
    "phase_5_deploy": 95,
    "phase_6_verification": 27
  },
  "compilation": {
    "rounds": 2,
    "initial_errors": 4,
    "final_errors": 0
  },
  "tests": {
    "total": 12,
    "passed": 12,
    "failed": 0,
    "skipped": 0,
    "route_coverage_pct": 100,
    "auth_coverage_pct": 100,
    "error_path_coverage_pct": 67
  },
  "verification": {
    "smoke_tests_total": 4,
    "smoke_tests_passed": 4,
    "tls_valid": true,
    "db_connected": true,
    "frontend_loads": true,
    "avg_response_ms": 23
  },
  "cost": {
    "total_api_calls": 14,
    "total_input_tokens": 89000,
    "total_output_tokens": 31000,
    "estimated_cost_usd": 1.42
  },
  "code_metrics": {
    "total_files": 9,
    "total_lines_rust": 420,
    "total_lines_sql": 28,
    "total_lines_tsx": 185,
    "total_lines_test": 210
  },
  "quality_score": 94
}
```

### Quality score (0-100)

| Component | Weight | Scoring |
|-----------|--------|---------|
| Compilation | 25 | 25 if clean on round 1, -5 per additional round, 0 if never clean |
| Tests | 25 | pass rate * coverage (route + auth + error path) |
| Verification | 25 | smoke tests passed + health checks (DB, TLS, frontend) |
| Efficiency | 25 | API cost relative to project size, fewer fix rounds = higher |

### Human-readable output

Printed to stdout at end of run:

```
forge build complete: link-shorty

  Blueprint:    8s    ok
  Generation:  24s    ok  9 files
  Compilation: 15s    ok  2 rounds (4 -> 0 errors)
  Tests:       18s    ok  12/12 passed
  Deploy:      95s    ok  https://link-shorty.example.com
  Verification:27s    ok  4/4 smoke tests

  Quality: 94/100
  Cost: $1.42 (14 API calls, 120k tokens)
  Total: 3m 7s

  Report: .forge/runs/2026-04-05-link-shorty-a1b2c3/report.json
```

## Future Enhancements (Not In Scope)

These patterns from hybrid-forge and the broader ecosystem can be pulled in over time:

- **Dual-backend routing** -- route struct/enum generation to cheaper/faster models, complex logic to Claude
- **Temperature tuning** -- per-file-type temperature control
- **OTLP/Langfuse integration** -- export telemetry to external observability
- **Conflict-free integration** -- for multi-agent collaborative builds on larger projects
- **Playbook-factory integration** -- forge as the Phase 2 engine in the factory pipeline
- **Multiple deploy targets** -- beyond fifty (Docker, fly.io, etc.)
- **Schema migration diffing** -- generate migration diffs from blueprint changes on subsequent runs
