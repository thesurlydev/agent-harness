# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

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
