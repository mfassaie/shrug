# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.2] - 2026-03-24

### Added
- Dynamic command engine generating CLI commands at runtime from OpenAPI 3.0.1 specs
- Three Atlassian Cloud products: Jira, Jira Software, Confluence (~925 operations)
- Two-phase CLI parsing (product routing, then dynamic command tree)
- Multi-profile authentication with OS keychain storage (macOS, Windows, Linux)
- OAuth 2.0 with PKCE and automatic token refresh
- Encrypted file fallback for credential storage
- Environment variable auth for CI/CD (`SHRUG_SITE`, `SHRUG_EMAIL`, `SHRUG_API_TOKEN`)
- Interactive first-run setup wizard (`shrug auth setup`)
- Generic HTTP executor with retries, exponential backoff, unified pagination
- Five output formats: JSON, table, YAML, CSV, plain text with TTY detection
- ADF terminal renderer (paragraphs, headings, lists, code blocks, inline marks)
- Markdown to ADF converter for rich text input
- JQL shorthand flags (`--project`, `--assignee me`, `--status`)
- Helper commands: `+create`, `+search`, `+transition` for Jira, `+create` for Confluence
- Shell completions for bash, zsh, fish, PowerShell with dynamic resource completion
- Binary spec caching with rkyv zero-copy deserialisation (<30ms warm startup)
- Serve-stale cache with background ETag refresh
- `--dry-run` mode, `-v`/`-vv`/`--trace` logging, `--fields` column selection
- Man page generation (`shrug _generate-man`)
- Smoke test suite (47 tests) against installed binary on PATH
- E2E test suite (70 tests) against live Atlassian Cloud
- cargo-dist release workflow for Linux (musl), macOS (Intel + ARM), Windows
- `scripts/release.sh` for version bumping and tagging

[0.7.2]: https://github.com/mfassaie/shrug/releases/tag/v0.7.2
