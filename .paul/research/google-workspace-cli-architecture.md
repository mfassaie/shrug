# Research: Google Workspace CLI Architecture

**Date:** 2026-03-21
**Source:** Local codebase at C:\Users\Falconer\Development\General\cli-main\cli-main
**Agent:** Explore

---

## Summary

The Google Workspace CLI (`gws`) is a **Rust** CLI that dynamically generates commands at runtime from Google Discovery Documents — the binary ships with zero API knowledge baked in.

## Technology Stack

- **Language:** Rust
- **CLI Framework:** `clap` v4 (derive macros)
- **HTTP:** `reqwest` v0.12 (async)
- **Auth:** `yup-oauth2` v12
- **Runtime:** `tokio` (async)
- **TUI:** `ratatui` (interactive setup)

## Core Architecture: Two-Phase Parsing

**Phase 1:** Extract service name from `argv[1]` (e.g., "drive")
**Phase 2:** Fetch Discovery Doc → build CLI tree → re-parse with dynamic tree

```
User: gws drive files list --params '{"pageSize": 10}'
  → Phase 1: Extract "drive" → resolve to ("drive", "v3")
  → Fetch/cache Discovery Doc from Google API
  → Phase 2: Build clap Command tree from resources/methods
  → Re-parse args against dynamic tree
  → Resolve to RestMethod (GET, path="files", params={...})
  → Build URL, authenticate, execute HTTP request
  → Format and output response
```

## Key Patterns

### 1. Discovery Document (NOT OpenAPI)
Google uses a proprietary REST Description format, not OpenAPI. Key differences:
- Resources are **nested** (not flat path strings)
- Parameters embedded in methods
- Schema refs use `"$ref": "MethodName"` not `#/components/schemas/`

### 2. Dynamic CLI Tree Builder (commands.rs)
- `build_cli(doc)` recursively walks `doc.resources` → creates nested clap subcommands
- Methods become leaf subcommands with generic args: `--params`, `--json`, `--upload`, `--page-all`
- Help text comes from Discovery Doc descriptions

### 3. Generic HTTP Executor (executor.rs, ~1100 LOC)
- RFC 6570 URI template substitution for path params
- Automatic separation of path vs query params
- Schema validation before dispatch
- Auto-pagination with `--page-all`
- Multipart upload support
- `--dry-run` support

### 4. Pluggable Helpers
Service-specific shortcuts via `Helper` trait:
- `gws gmail +send` (compose and send email)
- `gws drive +upload` (simplified upload)
- These bypass the dynamic system for better UX

### 5. Authentication (auth.rs)
Priority chain: env var token → encrypted creds → plaintext creds → ADC
- AES-256-GCM encryption for credentials at rest
- OS keyring for encryption key storage
- Interactive `gws auth setup` with TUI

### 6. Output Formatting
JSON (default), Table, YAML, CSV
- Intelligent array detection in responses
- NDJSON for paginated JSON output
- Headers-once for paginated CSV

### 7. MCP Server (mcp_server.rs)
Exposes discovered API methods as MCP tools for AI agents:
- `gws mcp -s drive,gmail` — expose specific services
- Full mode (1 tool/method) or Compact mode (1 tool/service)
- stdio JSON-RPC protocol

## Codebase Structure

| File | Responsibility | ~LOC |
|------|---------------|------|
| main.rs | Entry point, two-phase parsing | 400 |
| discovery.rs | Fetch, cache, deserialize Discovery Docs | 330 |
| commands.rs | Recursive CLI tree builder | 275 |
| executor.rs | Request building, validation, execution | 1100 |
| auth.rs | OAuth/ADC token acquisition | 300 |
| formatter.rs | Output formatting | 400 |
| schema.rs | Schema introspection command | 350 |
| helpers/*.rs | Service-specific shortcuts | 200+/each |
| mcp_server.rs | MCP server for AI agents | 500 |

## Key Takeaway for shrug

The binary knows nothing about APIs at compile time. Everything — commands, help text, validation, URL construction — derives from the spec document fetched at runtime. This means adding a new API or API version requires zero code changes.

---
*Research completed: 2026-03-21*
