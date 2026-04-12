# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.2.0] - 2026-04-12

### Added

- PRD enrichment runner with 3-round protocol (parallel enrichment, adversarial review, contract lock) and parallel agent dispatch across 11 specialist roles.
- 11 PRD enrichment agent prompt templates (`prompts/prd/*.md`) for product manager, API architect, Rust engineer, DBA, frontend architect, design lead, contract enforcer, security auditor, adversary, simplifier, and ops engineer.
- PRD types: `PrdIntentWrapper`, `PrdAgentRole`, `PrdCompletenessResponse`, `PrdVerifyResponse`, `PrdArtifactsResponse`, `VerificationReport`, `VerificationItem`.
- `--mode` CLI flag to switch between `code` (default) and `prd` modes.
- `--app-type` CLI flag to select agent pool (`api-only`, `full-stack`, `cli`).
- Orchestrator client methods for PRD endpoints: `get_prd_source`, `apply_prd_intent`, `get_prd_completeness`, `verify_prd`, `get_prd_artifacts`.
- PRD verification report (structural/behavioral/contract layers) printed after enrichment via `/a2a/prd-verify`.
- PRD artifact derivation (OpenAPI, test skeletons, verification checklist) written to `prd-artifacts/` when `--project-dir` is set, via `/a2a/prd-artifacts`.
- `agent_id` now sent in `complete_task` requests for audit trail alignment with conflict-free 0.6.0 SDK.

### Fixed

- Replace panics with resilient error handling in runner loop.
- Check HTTP status on `claim_task` and `complete_task` responses; return errors instead of silently succeeding on non-2xx.
- Handle markdown fence edge case in PRD intent parser where closing fence was not found.
- Align all PRD agent prompt intent schemas with `PrdIntent` enum fields (e.g., `metric` instead of `name`/`target` for `SetSuccessMetric`).
- Add `params` to `add_method` examples in code generation prompt.

## [0.1.3] - 2026-04-05

### Changed

- Externalized Claude prompts into `prompts/generate_intents.md` and `prompts/generate_fixes.md` for easier inspection and editing. Prompts are embedded at compile time via `include_str!`.

## [0.1.2] - 2026-04-04

### Added

- Compile-fix loop activated by `--project-dir` flag. After all tasks complete, syncs source to disk, runs `cargo check`, and sends errors back to Claude for up to `--max-fix-rounds` (default 3) rounds of automated fixes.
- `generate_fixes` method on `ClaudeClient` with a dedicated fix prompt targeting common Axum 0.8, tokio, and serde issues.
- `write_source_files` helper to sync orchestrator source (delimited by `// === path ===` markers) to the local filesystem.

## [0.1.1] - 2026-04-04

### Fixed

- Strip markdown code fences from Claude responses before JSON parsing.
- Skip failed intent-parse tasks instead of marking them complete (avoids blocking the DAG).

### Changed

- Expanded the intent-generation prompt with explicit examples for all available intent types (`add_use`, `add_struct`, `add_enum`, `add_function`, `add_impl`, `add_method`, `set_function_body`, `add_attribute`).
- Added Axum-specific rules to the prompt: prefer `tokio::sync::Mutex`, use `{param}` route syntax.

## [0.1.0] - 2026-04-04

### Added

- Initial release. CLI tool that polls a conflict-free orchestrator for ready tasks, generates AST-CRDT intents via the Claude API, and applies them back.
- `OrchestratorClient` for `/a2a/*` endpoints (tasks, source, intents).
- `ClaudeClient` for Anthropic Messages API.
- Runner loop: claim task, generate intents, apply, mark complete, exit when queue is empty.
- CLI via clap: `orchestrator_url`, `token`, `model`, `agent_name`.
