# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**shrug** is a dynamic CLI for Atlassian Cloud, supporting Jira, Jira Software, Confluence, BitBucket, and Service Management.

## Tech Stack

- Language: Rust (edition 2021)
- CLI framework: clap 4 (derive)
- Async runtime: tokio
- HTTP client: reqwest (rustls-tls)
- Error handling: thiserror
- Logging: tracing + tracing-subscriber

## Common Commands

- Build: `cargo build`
- Test: `cargo test`
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt`
- Run: `cargo run -- [args]`

## Key Directories

- `src/` — source code
- `.paul/` — PAUL project management
- `.github/` — CI workflows
