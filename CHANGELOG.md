# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-03-24

### Added
- Static command tree: `shrug <product> <entity> <verb>` with 37 entities and 140+ commands
- Three Atlassian Cloud products: Jira (16 entities, 63 commands), Jira Software (3 entities, 12 commands), Confluence (18 entities, 65 commands)
- Typed flags for every command (`--summary`, `--project`, `--type`, `--status`, `--label`, etc.)
- Three input tiers: typed flags, `--body`/`--body-file`, `--from-json` for full JSON control
- Three output formats: JSON, table, CSV with TTY detection
- Template generation (`shrug template`) for `--from-json` scaffolds
- Multi-profile authentication with OS keychain storage (macOS, Windows, Linux)
- OAuth 2.0 with PKCE and automatic token refresh
- Encrypted file fallback for credential storage
- Environment variable auth for CI/CD (`SHRUG_SITE`, `SHRUG_EMAIL`, `SHRUG_API_TOKEN`)
- ADF terminal renderer (paragraphs, headings, lists, code blocks, inline marks)
- Markdown to ADF converter for Jira rich text input
- Markdown to Confluence storage format converter
- JQL shorthand flags (`--project`, `--assignee me`, `--status`)
- Global flags: `--dry-run`, `--web`, `--limit`, `--output`, `--fields`, `--verbose`
- Claude Code skill installer (`shrug install-skill`)
- Shell installer (`install.sh`) and PowerShell installer (`install.ps1`)
- Homebrew formula and Scoop manifest for package manager distribution
- 413 unit tests, 142 mocked integration tests, 76 live E2E tests
- Cross-platform: Linux (musl), macOS (Intel + ARM), Windows

[1.0.0]: https://github.com/mfassaie/shrug/releases/tag/v1.0.0
