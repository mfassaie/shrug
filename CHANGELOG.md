# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2026-03-23

### Added
- Smoke test suite (47 tests) running against installed binary on PATH
- SmokeRunner harness with offline/online modes and env var isolation
- Static command tests for profile CRUD, auth, cache, completions
- Global flag tests for all output formats, colour modes, verbose/trace
- Help message snapshots using insta golden-file testing
- Error message and exit code validation tests
- Live API CRUD smoke tests for Jira issues and Confluence pages
- Release script (`scripts/release.sh`) for version bumping and tagging

## [0.3.0] - 2026-03-23

### Added
- Jira Platform: 20 entity groups tested with full CRUD
- Jira Software: boards, sprints, epics, backlog operations
- Confluence: 20 entity groups (pages, blog posts, spaces, folders, whiteboards)
- Pagination limit verification, verbose/trace logging tests
- ADF comment round-trip test (create, read, verify content)

### Fixed
- `+search` updated from deprecated to enhanced search API
- Global shorthand flags (`--project`, `--assignee`, `--status`) now forwarded to helpers
- 7 clippy warnings eliminated

## [0.2.0] - 2026-03-23

### Added
- E2E test harness with ShrugRunner, ResourceTracker, skip_unless_e2e! macro
- Three-tier spec loading: binary cache, JSON cache, bundled fallback
- `shrug cache refresh` command for manual spec refresh
- 40 E2E tests against live Atlassian Cloud
- Confluence v2 API support (212 operations, up from 130 in v1)
- All 5 output format tests, dry-run mode test

### Fixed
- URL resolution: credential site now takes priority over spec placeholder
- URL path prefix preserved for APIs with path prefixes (Confluence /wiki/api/v2)
- Global `--json`/`--output` flags no longer captured as trailing args

## [0.1.0] - 2026-03-21

### Added
- Dynamic command engine: OpenAPI 3.0.1 and Swagger 2.0 parsers
- Two-phase CLI parsing (product routing, then dynamic command tree)
- Five Atlassian Cloud products: Jira, Jira Software, Confluence, Bitbucket, JSM
- Multi-profile authentication with OS keychain storage
- OAuth 2.0 with PKCE and automatic token refresh
- Encrypted file fallback for credential storage
- Generic HTTP executor with retries, backoff, unified pagination
- Output formatters: JSON, table, YAML, CSV, plain text
- ADF terminal renderer (paragraphs, headings, lists, code, marks)
- Markdown to ADF converter for rich text input
- JQL shorthand builder (`--project`, `--assignee`, `--status`)
- Helper commands: `+create`, `+search`, `+transition`
- Shell completions for bash, zsh, fish, PowerShell
- cargo-dist release workflow for Linux, macOS, Windows

[0.5.0]: https://github.com/mfassaie/shrug/compare/v0.3.0...v0.5.0
[0.3.0]: https://github.com/mfassaie/shrug/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/mfassaie/shrug/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/mfassaie/shrug/releases/tag/v0.1.0
