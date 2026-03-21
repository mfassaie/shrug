# Roadmap: shrug

## Overview

A dynamic CLI for Atlassian Cloud — commands generated at runtime from OpenAPI specs, supporting Jira, Jira Software, Confluence, BitBucket, and Service Management. Built in Rust, inspired by Google Workspace CLI architecture.

## Current Milestone

**v0.1 MVP** (v0.1.0)
Status: In progress
Phases: 5 of 8 complete

## Phases

| Phase | Name | Plans | Status | Completed |
|-------|------|-------|--------|-----------|
| 1 | Project Foundation | 3 | ✅ Complete | 2026-03-21 |
| 2 | OpenAPI Spec Engine | 4 | ✅ Complete | 2026-03-21 |
| 3 | Dynamic Command Tree | 2 | ✅ Complete | 2026-03-21 |
| 4 | Authentication & Profiles | 3 | ✅ Complete | 2026-03-21 |
| 5 | Generic HTTP Executor | 4 | ✅ Complete | 2026-03-21 |
| 6 | Output & Formatting | 2 | Not started | - |
| 7 | Helper Commands & ADF | 3 | Not started | - |
| 8 | Distribution & Polish | 3 | Not started | - |

## Phase Details

### Phase 1: Project Foundation

**Goal:** Rust project scaffold with core infrastructure — config, errors, logging, CI
**Depends on:** Nothing (first phase)
**Research:** Unlikely (established patterns)

**Scope:**
- Cargo workspace structure and core dependencies
- Config system (TOML, layered precedence, platform paths)
- Error types, exit codes, logging framework
- CI scaffold (GitHub Actions: Linux/macOS/Windows)

**Plans:**
- [x] 01-01: Cargo project scaffold, dependencies, and project structure
- [x] 01-02: Config system with TOML, layered precedence, and platform paths
- [x] 01-03: Enhanced logging, Ctrl+C handling, and config debug dump

### Phase 2: OpenAPI Spec Engine

**Goal:** Parse, cache, and serve OpenAPI 3.0.1 and Swagger 2.0 specs with fast startup
**Depends on:** Phase 1 (project structure, error types, config for cache paths)
**Research:** Unlikely (OpenAPI format well-documented in research)

**Scope:**
- OpenAPI 3.0.1 parser (paths, operations, parameters, tags)
- Swagger 2.0 parser or conversion layer (BitBucket)
- Spec caching: JSON + rkyv binary cache with 24h TTL
- Pre-bundled fallback specs in binary
- Background refresh with ETag
- Spec conformance test suite (auto-generated from specs)

**Plans:**
- [x] 02-01: OpenAPI 3.0.1 spec parser and data model
- [x] 02-02: Swagger 2.0 parser / conversion layer for BitBucket
- [x] 02-03: Spec caching (JSON, bundled fallback, version detection)
- [x] 02-04: Spec analysis & conformance test suite (URL building, pagination detection, param validation)

### Phase 3: Dynamic Command Tree

**Goal:** Two-phase CLI parsing that builds commands from specs at runtime
**Depends on:** Phase 2 (spec parser provides the data model)
**Research:** Unlikely (gws architecture well-documented in research)

**Scope:**
- Product routing (argv[1] → spec selection)
- Tags → command groups, operationId → leaf commands
- Parameter → flag generation
- Help text from spec descriptions

**Plans:**
- [x] 03-01: Two-phase parsing and product routing
- [x] 03-02: Command tree builder (tags, operations, parameters, help)

### Phase 4: Authentication & Profiles

**Goal:** Multi-profile auth with keychain storage, OAuth2, and CI/CD env vars
**Depends on:** Phase 1 (config system, platform paths, error types)
**Research:** Unlikely (keyring crate and Atlassian auth well-documented)

**Scope:**
- Profile CRUD (create/use/list/show/delete)
- Keychain integration (macOS/Windows/Linux)
- Basic Auth (email + API token) and OAuth 2.0 with refresh
- Encrypted file fallback
- Environment variable override
- Interactive setup wizard

**Plans:**
- [x] 04-01: Profile management and config integration
- [x] 04-02: Keychain credential storage with encrypted file fallback
- [x] 04-03: OAuth 2.0 flow, token refresh, and interactive setup wizard

### Phase 5: Generic HTTP Executor

**Goal:** Execute any spec-defined API call with auth, pagination, retries, and validation
**Depends on:** Phase 3 (command tree resolves to method), Phase 4 (auth provides tokens)
**Research:** Unlikely (patterns established by gws research)

**Scope:**
- URL template substitution and query param injection
- Auth header injection from active profile
- Request body handling (--json flag)
- Rate limit detection + exponential backoff
- Unified pagination iterator
- --dry-run support
- Connection pooling
- Quirks registry for endpoint-specific behavior not captured in specs

**Plans:**
- [x] 05-01: URL building, request construction, and basic execution
- [x] 05-02: Rate limiting, retries, and error response handling
- [x] 05-03: Unified pagination iterator (offset, cursor, link-based)
- [x] 05-04: Quirks registry (operationId → special headers, non-spec behaviors)

### Phase 6: Output & Formatting

**Goal:** Multiple output formats with TTY detection and ADF terminal rendering
**Depends on:** Phase 5 (executor provides API responses to format)
**Research:** Unlikely (standard CLI patterns)

**Scope:**
- Table, JSON, YAML, CSV, plain text formatters
- TTY detection, NO_COLOR, pager integration
- --fields column selection
- ADF → terminal rendering

**Plans:**
- [ ] 06-01: Output formatters (table, JSON, YAML, CSV, plain) with TTY detection
- [ ] 06-02: ADF terminal rendering, pager integration, and --fields selection

### Phase 7: Helper Commands & ADF

**Goal:** UX shortcuts for common operations and Markdown → ADF input conversion
**Depends on:** Phase 5 (executor for API calls), Phase 6 (formatters for output)
**Research:** Likely (ADF conversion specifics, JQL shorthand design)
**Research topics:** Markdown-to-ADF Rust implementation, custom field metadata API

**Scope:**
- Markdown → ADF converter for input
- JQL shorthand flags (--project, --assignee, --status)
- +create, +search, +transition helper commands
- Custom field name resolution
- User lookup (display name → accountId)
- Shell completions

**Plans:**
- [ ] 07-01: Markdown → ADF converter and JQL shorthand flags
- [ ] 07-02: Helper commands (+create, +search, +transition)
- [ ] 07-03: Shell completions and field/user resolution caches

### Phase 8: Distribution & Polish

**Goal:** Release automation, packaging, documentation, and test validation
**Depends on:** All prior phases
**Research:** Unlikely (cargo-dist well-documented)

**Scope:**
- cargo-dist release automation
- Homebrew tap, Scoop manifest
- First-run experience polish
- Mock-based integration tests (httpmock/wiremock with recorded responses)
- E2E smoke tests against live Atlassian Cloud (on-demand CI workflow)
- Performance benchmarking

**Plans:**
- [ ] 08-01: cargo-dist release pipeline, Homebrew tap, and Scoop manifest
- [ ] 08-02: Mock-based integration tests with recorded API responses
- [ ] 08-03: On-demand E2E smoke tests against live Atlassian Cloud, performance benchmarks, and first-run polish

---
*Roadmap created: 2026-03-21*
*Last updated: 2026-03-21 after Phase 5*
