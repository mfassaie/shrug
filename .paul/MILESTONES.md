# Milestones

Completed milestone log for this project.

| Milestone | Completed | Duration | Stats |
|-----------|-----------|----------|-------|
| v0.1 MVP | 2026-03-21 | 1 day | 8 phases, 24 plans |

---

## v0.1 MVP

**Completed:** 2026-03-21
**Duration:** 1 day
**Version:** 0.1.0

### Stats

| Metric | Value |
|--------|-------|
| Phases | 8 |
| Plans | 24 |
| Files changed | ~35 unique |
| Tests | 395 (388 unit + 7 integration) |
| Enterprise audits | 24 (one per plan) |

### Key Accomplishments

- **Dynamic command engine:** OpenAPI 3.0.1 and Swagger 2.0 parsers generate CLI commands at runtime from Atlassian specs. Two-phase parsing routes to product, then builds the command tree. Covers all 5 Atlassian Cloud products (~1,250 operations).
- **Authentication system:** Multi-profile auth with OS keychain storage, OAuth 2.0 with PKCE and token refresh, encrypted file fallback, environment variable support for CI/CD, and an interactive setup wizard.
- **HTTP executor:** Generic executor handles URL template substitution, request construction, retries with exponential backoff and jitter, unified pagination (offset/cursor/link-based), and a quirks registry for endpoint-specific behaviours.
- **Output formatting:** Table, JSON, YAML, CSV, and plain text formatters with TTY detection, NO_COLOR support, pager integration, --fields column selection, and ADF terminal rendering (paragraphs, headings, lists, code blocks, inline marks).
- **Helper commands:** Markdown-to-ADF converter, JQL shorthand builder (--project, --assignee, --status), +create/+search/+transition shortcuts, field name and user display name resolution with site-scoped TTL caches, shell completions for bash/zsh/fish/PowerShell.
- **Distribution pipeline:** cargo-dist release workflow targeting Linux (musl), macOS, and Windows. Homebrew tap and Scoop manifest templates. Mock-based integration tests with recorded API responses. On-demand E2E smoke test workflow against live Atlassian Cloud.
- **Error polish:** Actionable error remediation hints on every error variant, first-run detection for unconfigured users, performance benchmarks (spec parsing ~0.19ms, command tree ~0.007ms per iteration).

### Key Decisions

| Decision | Phase | Rationale |
|----------|-------|-----------|
| Rust + clap + tokio + reqwest stack | 1 | Cross-platform static binary, performance, ecosystem |
| Dynamic command gen from OpenAPI specs | 1 | Automatic API coverage, no code changes for new endpoints |
| Purpose-built spec model (not full OpenAPI) | 2 | Only CLI-needed fields, keeps parser simple and extensible |
| ShrugConfigPartial merge pattern | 1 | Layered config with Option fields that merge without silently resetting |
| rkyv for spec caching (planned) | 2 | Zero-copy deserialisation for fast startup |
| Non-panicking Ctrl+C handler | 1 | if-let pattern prevents panic on handler setup failure |
| .default file pattern for profiles | 4 | Simple, atomic profile switching |
| Keychain-first credential storage | 4 | No plaintext tokens, OS-level security |
| Static quirks registry | 5 | Endpoint-specific headers not in specs (CSRF bypass) |
| remediation() as separate method | 8 | Keeps error Display trait stable, hints are optional |

---
*Milestones log — Updated on milestone completion*
