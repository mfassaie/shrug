# Project: shrug

## What This Is

shrug is a dynamic CLI tool for Atlassian Cloud, supporting Jira, Jira Software, Confluence, BitBucket, and Service Management. Commands are generated at runtime from Atlassian's OpenAPI specifications — the binary ships with no hardcoded API knowledge. Inspired by the Google Workspace CLI (`gws`) architecture.

## Core Value

Users and AI agents can interact with all Atlassian Cloud products from the command line without context-switching to a browser, with commands that stay current as Atlassian evolves their APIs.

## Current State

| Attribute | Value |
|-----------|-------|
| Version | 0.5.0 |
| Status | v0.5 complete |
| Last Updated | 2026-03-23 |
| Phase | All phases complete (v0.1 + v0.2 + v0.3 + v0.4 + v0.5) |

## Requirements

### Must Have (MVP)

**Dynamic Command Engine**
- [x] Two-phase CLI parsing: extract product → load spec → build command tree → execute — Phase 3
- [x] OpenAPI 3.0.1 spec parser for Jira, Jira Software, Confluence, Service Management — Phase 2
- [x] Swagger 2.0 spec parser for BitBucket (direct-to-ApiSpec conversion) — Phase 2
- [x] Runtime command tree generation from specs (tags → command groups, operationId → commands) — Phase 3
- [x] Generic HTTP executor with URL template substitution, parameter validation — Phase 5
- [x] Spec caching with 24h TTL — Phase 2 (manual refresh command deferred to Phase 3)
- [x] Pre-bundled fallback specs in binary for offline/first-run — Phase 2

**Authentication & Profiles**
- [x] API token + email auth (Basic Auth) — primary method — Phase 4
- [x] OAuth 2.0 (3LO) with automatic token refresh — Phase 4
- [x] OS keychain credential storage (macOS Keychain, Windows Credential Manager, Linux Secret Service) — Phase 4
- [x] Encrypted file fallback when keychain unavailable — Phase 4
- [x] Multi-profile support (`shrug profile create/use/list/show`) — Phase 4
- [x] Per-command profile override (`--profile staging`) — Phase 4
- [x] Environment variable auth for CI/CD (`SHRUG_API_TOKEN`, `SHRUG_EMAIL`, `SHRUG_SITE`) — Phase 4
- [x] Interactive first-run setup wizard (`shrug auth setup`) — Phase 4

**Output & Formatting**
- [x] Multiple output formats: JSON (default for pipes), table (default for TTY), YAML, CSV, plain — Phase 6
- [x] TTY detection for automatic format/color selection — Phase 6
- [x] `NO_COLOR` and `--color=auto|always|never` support — Phase 6
- [x] Pager integration for long output (`$PAGER`, `--no-pager`) — Phase 6
- [x] `--fields` for column selection in table output — Phase 6

**Atlassian Data Handling**
- [x] ADF (Atlassian Document Format) input: accept Markdown/plain text → convert to ADF — Phase 7
- [x] ADF output: render as plain text or ANSI-formatted terminal output — Phase 6
- [x] JQL support via `--jql` flag for raw queries — Phase 7
- [x] Unified pagination iterator (offset-based, cursor-based, link-based) — Phase 5
- [x] Auto-pagination with `--page-all` and `--limit` — Phase 5

**Error Handling & Resilience**
- [x] Structured exit codes (0=ok, 2=usage, 3=auth, 4=not-found, 5=forbidden, 10=rate-limited, 11=network, 12=server) — Phase 1
- [x] Rate limit handling: parse `Retry-After`, exponential backoff with jitter, max 4 retries — Phase 5
- [x] Actionable error messages (what happened → why → what to do) — Phase 8

**Cross-Platform**
- [x] Windows, macOS, Linux support — Phase 8 (release workflow targets all 3)
- [x] Platform-correct config/cache paths via `directories` crate — Phase 1
- [x] Static binary distribution (musl for Linux) — Phase 8

### Should Have

**Helper Commands (UX shortcuts)**
- [x] `shrug jira +create` — simplified issue creation with Markdown description — Phase 7
- [x] `shrug jira +search` — shorthand flags (`--project`, `--assignee me`, `--status`) — Phase 7
- [x] `shrug jira +transition` — resolve transition by name, not ID — Phase 7
- [x] `shrug confluence +create` — create page from Markdown file — Phase 19
- [x] Custom field name resolution (human name → customfield_ID via metadata cache) — Phase 7
- [x] User lookup by display name → accountId resolution — Phase 7

**Configuration**
- [x] TOML config with layered precedence (flags → env → project → user → defaults) — Phase 1
- [x] Project-level config (`.shrug.toml` in cwd/git root) — Phase 1
- [x] Configurable defaults: output format, page size, default project — Phase 1

**Shell Completions**
- [x] `shrug completions <shell>` for bash, zsh, fish, PowerShell — Phase 7
- [x] Dynamic completions for project keys, issue keys (cached, short TTL) — Phase 20

**Logging & Debugging**
- [x] `-v` (info), `-vv` (debug), `--trace` (full request/response with masked secrets) — Phase 1 (request logging placeholder, actual in Phase 5)
- [x] `--dry-run` to show request without sending — Phase 5
- [x] All debug output to stderr — Phase 1

**Performance**
- [x] Binary spec cache with rkyv zero-copy deserialization (<30ms warm startup) — Phase 18
- [x] Lazy per-product spec loading — already implemented (SpecLoader loads per-product)
- [x] Serve-stale cache with background ETag refresh — Phase 18
- [x] Connection pooling via reqwest Client reuse — already implemented (shared Client in main.rs)

### Nice to Have

**MCP Server**
- [ ] `shrug mcp` — expose Atlassian APIs as MCP tools for AI agents
- [ ] Full mode (1 tool/method) and compact mode (1 tool/product)
- [ ] stdio JSON-RPC protocol

**Advanced Features**
- [ ] `shrug schema <product> <resource>` — introspect API schemas
- [ ] Batch operations with configurable concurrency and rate awareness
- [ ] Watch/poll commands (`shrug issue watch KEY --poll 30s`)
- [ ] Webhook-triggered command execution
- [ ] Interactive TUI for complex operations (ratatui)
- [ ] Plugin/extension system for custom commands

**Distribution**
- [x] Homebrew tap (macOS/Linux) — Phase 8 (template formula)
- [x] Scoop manifest (Windows) — Phase 8 (template manifest)
- [ ] WinGet package
- [ ] `cargo binstall` support
- [x] GitHub Releases with automated builds via `cargo-dist` — Phase 8
- [ ] macOS notarization, Windows code signing

### Out of Scope

- Atlassian Server/Data Center support — Cloud only
- Wiki markup support — ADF only (wiki markup deprecated in Cloud)
- GUI or web interface — CLI and MCP only
- Forge/Connect app framework — shrug is a client tool, not an Atlassian app
- Real-time websocket connections — polling only

## Target Users

**Primary:** Developers and DevOps engineers using Atlassian Cloud
- Comfortable with command line tools
- Want to automate workflows, script Jira/Confluence operations
- Manage multiple Atlassian sites/projects

**Secondary:** AI agents and automation systems
- Need machine-readable output (JSON)
- MCP integration for LLM-driven workflows
- CI/CD pipelines interacting with Atlassian

## Context

**Business Context:**
- No widely-adopted open-source CLI exists for Atlassian Cloud (unlike `gh` for GitHub)
- Atlassian's API surface is large (~1,250 operations across 5 products) — manual CLI wrapping is infeasible
- Dynamic generation from OpenAPI specs means shrug automatically supports new API endpoints

**Technical Context:**
- Modeled after Google Workspace CLI (`gws`) — proven dynamic CLI architecture in Rust
- Atlassian provides OpenAPI 3.0.1 specs for 4/5 products; BitBucket uses Swagger 2.0
- Atlassian Cloud uses OAuth2 + Basic Auth (email + API token)
- Rate limiting is burst-based for API token traffic (not points-based)

## Constraints

### Technical Constraints
- Must parse OpenAPI 3.0.1 specs at runtime (not code-generated)
- Must handle Swagger 2.0 for BitBucket (conversion or dual parser)
- Jira Platform spec is 2.47MB — requires caching and optimization
- Warm startup target: <30ms; cold with bundled specs: <50ms
- Must use `rustls` (not `native-tls`) to avoid 100ms cold-start penalty

### Business Constraints
- Atlassian Cloud only — no Server/Data Center
- Must respect Atlassian rate limits (burst: 100 GET/POST, 50 PUT/DELETE per second)
- Credentials must never be stored in plaintext

### Compliance Constraints
- Secrets in OS keychain or encrypted file only
- `--trace` output must auto-mask tokens/credentials
- No credentials accepted via CLI flags (visible in process list)

## Key Decisions

| Decision | Rationale | Date | Status |
|----------|-----------|------|--------|
| Rust as language | Cross-platform static binary, performance for spec parsing, ecosystem (clap, reqwest, tokio) | 2026-03-21 | Active |
| Dynamic command generation | Automatic API coverage, no code changes for new endpoints, proven by gws | 2026-03-21 | Active |
| Two-phase CLI parsing | Only load spec for requested product, fast startup | 2026-03-21 | Active |
| TOML for config | Human-readable, comment support, type-safe, Rust ecosystem standard | 2026-03-21 | Active |
| rkyv for spec caching | Zero-copy deserialization (0.001ms vs 5ms JSON), critical for startup time | 2026-03-21 | Active |
| ADF only (no wiki markup) | Wiki markup deprecated in Cloud, ADF is the future | 2026-03-21 | Active |

## Success Metrics

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Warm startup time | <30ms | ~0.2ms (test fixture) | Needs full-spec measurement |
| Cold startup (bundled) | <50ms | Not measured | Needs measurement |
| Atlassian products supported | 5 (Jira, JSW, Confluence, BB, JSM) | 5 | Achieved — v0.1 MVP |
| API coverage | 100% of OpenAPI-defined operations | 100% (dynamic) | Achieved — v0.1 MVP |
| Platform support | Windows + macOS + Linux | 3 | Achieved — v0.1 MVP |

## Tech Stack

| Layer | Technology | Notes |
|-------|------------|-------|
| Language | Rust | Cross-platform, performance, ecosystem |
| CLI Framework | clap v4 (derive) | Industry standard, shell completions via clap_complete |
| Async Runtime | tokio | Required by reqwest, proven |
| HTTP Client | reqwest v0.12 (rustls-tls) | Connection pooling, HTTP/2, no native-tls penalty |
| Config | toml + serde | Layered config, TOML format |
| Paths | directories v5 | Platform-correct config/cache/data dirs |
| Credentials | keyring v3 | macOS/Windows/Linux keychain |
| Encryption | aes-gcm + argon2 | Fallback credential storage |
| Spec Caching | rkyv | Zero-copy deserialization for fast startup |
| Output Tables | comfy-table or tabled | Terminal table rendering |
| Colors | owo-colors + enable-ansi-support | Cross-platform ANSI color |
| Prompts | dialoguer | Interactive input, password prompts |
| Progress | indicatif | Spinners, progress bars |
| Errors | anyhow + thiserror | Application vs library errors |
| Logging | tracing + tracing-subscriber | Structured, leveled logging |
| Signals | ctrlc | Cross-platform Ctrl+C handling |
| Testing | assert_cmd + predicates + insta | CLI integration + snapshot tests |
| Release | cargo-dist + cargo-release | Automated multi-platform builds |

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        USER / AI AGENT                          │
│                                                                 │
│  shrug jira issues list --project TEST --output table           │
│  shrug confluence pages create --space DEV --file page.md       │
│  shrug mcp -s jira,confluence    (MCP server mode)              │
└─────────────────────┬───────────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────────┐
│                    PHASE 1: ROUTING                              │
│                                                                 │
│  Parse argv[1] → resolve product name + version                 │
│  "jira" → ("jira-platform", "v3")                               │
│  "confluence" → ("confluence", "v2")                            │
│  "bitbucket" → ("bitbucket", "2.0")                             │
│  Also handles: --version, --help, auth, profile, cache, mcp    │
└─────────────────────┬───────────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────────┐
│                 SPEC LOADER & CACHE                              │
│                                                                 │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────────────┐     │
│  │ rkyv Binary  │→│ Raw JSON     │→│ Bundled Fallback    │     │
│  │ Cache (<1ms) │  │ Cache (5ms)  │  │ in Binary (50ms)   │     │
│  └─────────────┘  └──────────────┘  └────────────────────┘     │
│         ↑                ↑                     ↑                │
│         └────── Cache Miss ──────── Network Fetch ──────┐      │
│                                                          │      │
│  Background: ETag check after command execution          │      │
│  Manual: shrug cache refresh                             │      │
└─────────────────────┬───────────────────────────────────────────┘
                      │ OpenAPI Spec (parsed)
┌─────────────────────▼───────────────────────────────────────────┐
│              PHASE 2: DYNAMIC COMMAND TREE                       │
│                                                                 │
│  OpenAPI spec → Command tree:                                   │
│    Tags → Command groups (e.g., "issues", "projects")           │
│    operationId → Leaf commands (e.g., "createIssue", "getIssue")│
│    Parameters → Flags (--issueIdOrKey, --maxResults)            │
│    Request body → --json flag                                   │
│                                                                 │
│  + Helper commands injected (+create, +search, +transition)     │
│  Re-parse argv against dynamic tree                             │
└─────────────────────┬───────────────────────────────────────────┘
                      │ Resolved: method + params + body
┌─────────────────────▼───────────────────────────────────────────┐
│                   AUTH LAYER                                     │
│                                                                 │
│  Profile resolution: --profile flag → env → config default      │
│  Credential lookup: keychain → encrypted file → env vars        │
│  Token type: API token (Basic) or OAuth2 (Bearer + refresh)     │
└─────────────────────┬───────────────────────────────────────────┘
                      │ Authenticated request
┌─────────────────────▼───────────────────────────────────────────┐
│                GENERIC HTTP EXECUTOR                             │
│                                                                 │
│  1. Build URL (base + path template substitution)               │
│  2. Separate path params from query params                      │
│  3. Validate params against spec                                │
│  4. Convert Markdown → ADF for rich text fields                 │
│  5. Build HTTP request (method, headers, body)                  │
│  6. Execute (with retry on 429/5xx, exponential backoff)        │
│  7. Handle pagination (unified iterator)                        │
│  8. Return response                                             │
│                                                                 │
│  Shared reqwest::Client (connection pooling, HTTP/2)            │
└─────────────────────┬───────────────────────────────────────────┘
                      │ API response (JSON)
┌─────────────────────▼───────────────────────────────────────────┐
│                 OUTPUT FORMATTER                                 │
│                                                                 │
│  ┌───────┐ ┌───────┐ ┌──────┐ ┌─────┐ ┌───────┐              │
│  │ Table │ │ JSON  │ │ YAML │ │ CSV │ │ Plain │              │
│  └───────┘ └───────┘ └──────┘ └─────┘ └───────┘              │
│                                                                 │
│  ADF → terminal rendering (bold, links, lists)                  │
│  Pager integration for long output                              │
│  --fields for column selection                                  │
│  TTY detection for auto-format                                  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                 CROSS-CUTTING CONCERNS                           │
│                                                                 │
│  Config:     TOML, layered (flags→env→project→user→defaults)   │
│  Profiles:   Multi-site, per-profile credentials                │
│  Logging:    tracing (-v, -vv, --trace), secrets masked         │
│  Errors:     Structured (what→why→action), typed exit codes     │
│  Signals:    Graceful Ctrl+C shutdown                           │
│  Completions: bash/zsh/fish/PowerShell, dynamic for resources   │
└─────────────────────────────────────────────────────────────────┘
```

## Supported Products & Spec Sources

| Product | CLI Prefix | Spec Format | Spec URL | ~Operations |
|---------|-----------|-------------|----------|-------------|
| Jira Platform | `shrug jira` | OpenAPI 3.0.1 | `dac-static.atlassian.com/cloud/jira/platform/swagger-v3.v3.json` | 620 |
| Jira Software | `shrug jira-software` | OpenAPI 3.0.1 | `dac-static.atlassian.com/cloud/jira/software/swagger.v3.json` | 95 |
| Confluence | `shrug confluence` | OpenAPI 3.0.1 | `dac-static.atlassian.com/cloud/confluence/swagger.v3.json` | 130 |
| Jira Service Mgmt | `shrug jsm` | OpenAPI 3.0.1 | `dac-static.atlassian.com/cloud/jira/service-desk/swagger.v3.json` | 70 |
| BitBucket | `shrug bitbucket` | Swagger 2.0 | `bitbucket.org/api/swagger.json` | 335 |

## Links

| Resource | URL |
|----------|-----|
| Repository | (this repo) |
| Reference CLI (gws) | https://github.com/googleworkspace/cli |
| Jira OpenAPI Spec | https://dac-static.atlassian.com/cloud/jira/platform/swagger-v3.v3.json |
| Atlassian Auth Docs | https://developer.atlassian.com/cloud/jira/platform/rest/v3/intro/ |
| ADF Spec | https://developer.atlassian.com/cloud/jira/platform/apis/document/structure/ |

---
*PROJECT.md — Updated when requirements or context change*
*Last updated: 2026-03-23 after Phase 20 (Dynamic Completions) — v0.4 milestone complete*
