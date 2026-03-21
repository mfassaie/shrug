# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**shrug** is a dynamic CLI for Atlassian Cloud, supporting Jira, Jira Software, Confluence, BitBucket, and Service Management. Commands are generated at runtime from OpenAPI/Swagger specs, not hardcoded.

## Tech Stack

- Language: Rust (edition 2021)
- CLI framework: clap 4 (derive)
- Async runtime: tokio
- HTTP client: reqwest (rustls-tls)
- Error handling: thiserror
- Logging: tracing + tracing-subscriber
- Crypto: aes-gcm, argon2, sha2 (credential encryption)
- Keychain: keyring crate (OS credential store)
- Serialisation: serde + serde_json, toml

## Common Commands

- Build: `cargo build`
- Test: `cargo test`
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt`
- Run: `cargo run -- [args]`

## Source Architecture

The codebase follows a modular structure. Each directory maps to a completed phase.

- `src/config.rs` — Layered TOML config with env var overrides, platform-aware paths
- `src/error.rs` — thiserror-based error types with structured exit codes (`src/exit_codes.rs`)
- `src/logging.rs` — tracing setup with env-filter
- `src/cli.rs` — Top-level clap definition, two-phase argument parsing
- `src/spec/` — OpenAPI 3.0.1 and Swagger 2.0 parsing, binary caching (rkyv planned), spec analysis
  - `model.rs` — Purpose-built spec model (not full OpenAPI, only CLI-needed fields)
  - `parser.rs` — OpenAPI 3.0.1 parser
  - `swagger.rs` — Swagger 2.0 parser (BitBucket compatibility)
  - `cache.rs` — JSON spec caching with version detection
  - `analysis.rs` — URL building, pagination detection, parameter validation
  - `registry.rs` — Product-to-spec mapping
- `src/cmd/` — Dynamic command tree built from specs at runtime
  - `router.rs` — Product routing (argv[1] → spec selection)
  - `tree.rs` — Tags → command groups, operationId → leaf commands, parameters → flags
- `src/auth/` — Multi-profile authentication with keychain storage
  - `profile.rs` — Profile CRUD, .default file pattern
  - `credentials.rs` — Keychain + encrypted file fallback
  - `oauth.rs` — OAuth 2.0 with PKCE, token refresh, localhost callback
- `src/executor.rs` — Generic HTTP executor with retries, pagination, quirk injection
- `src/quirks.rs` — Static quirks registry for endpoint-specific headers (CSRF bypass)
- `src/output.rs` — Output formatters (JSON, table, YAML, CSV, plain), TTY detection, pager, --fields
- `src/adf.rs` — Atlassian Document Format terminal renderer (paragraph, heading, list, code, marks)
- `src/markdown_to_adf.rs` — Markdown → ADF converter using pulldown-cmark (input direction)
- `src/jql.rs` — JQL shorthand builder (--project, --assignee, --status → JQL query)
- `src/helpers.rs` — Helper commands (+create, +search, +transition) with direct HTTP
- `src/completions.rs` — Shell completion generator (bash, zsh, fish, PowerShell) via clap_complete
- `src/resolve.rs` — Field name and user display name resolution with site-scoped TTL cache

## Key Design Patterns

- **ShrugConfigPartial merge pattern**: Layered config uses Option fields that merge without silently resetting values
- **Two-phase CLI parsing**: First pass routes to product, second pass builds commands from spec
- **Purpose-built spec model**: Only stores fields needed for CLI generation, not full OpenAPI

## Key Directories

- `src/` — source code
- `.paul/` — PAUL project management
- `.github/` — CI workflows
