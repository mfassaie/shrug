# Milestones

Completed milestone log for this project.

| Milestone | Completed | Duration | Stats |
|-----------|-----------|----------|-------|
| v0.1 MVP | 2026-03-21 | 1 day | 8 phases, 24 plans |
| v0.2 E2E Validation | 2026-03-23 | 1 day | 4 phases, 8 plans |
| v0.3 Test Coverage & Entity Expansion | 2026-03-23 | 1 day | 5 phases, 9 plans |

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

## v0.2 E2E Validation

**Completed:** 2026-03-23
**Duration:** 1 day
**Version:** 0.2.0

### Stats

| Metric | Value |
|--------|-------|
| Phases | 4 |
| Plans | 8 |
| E2E tests | 40 |
| Total tests | 435 (388 unit + 7 integration + 40 E2E) |
| Bugs found & fixed | 3 |
| Bugs documented | 2 |

### Key Accomplishments

- **E2E test harness:** ShrugRunner wrapping assert_cmd with env var auth, skip_unless_e2e!() macro, ResourceTracker with Drop for panic-safe cleanup, configurable rate limiting and command timeout. 40 tests run against live Atlassian Cloud in ~41 seconds.
- **Network spec fetching:** Three-tier SpecLoader (cache → network → bundled), `shrug cache refresh` command. Downloads real OpenAPI specs from Atlassian CDN (1,227 operations across 5 products).
- **Jira CRUD tests:** Full lifecycle tests for Issues, Comments, Worklogs, Filters, Dashboards, Versions, Components. Read-only tests for Projects, Statuses, Priorities, Resolutions, Issue Types, Fields, Search.
- **Confluence v2 upgrade:** Switched from v1 API spec (130 ops) to v2 (212 ops). Full page CRUD now available. Tests cover Page lifecycle, Spaces, Blog Posts, Labels.
- **CLI feature tests:** All 5 output formats (JSON, table, YAML, CSV, plain), dry-run mode, error remediation hints.

### Bugs Found & Fixed

| Bug | Fix | Impact |
|-----|-----|--------|
| resolve_base_url() used spec placeholder URL instead of user's site | Credential site takes priority | All API calls went to wrong domain |
| URL path prefix lost when using credential site | Extract and preserve path from spec server URL | Confluence v2 API paths broken without /wiki/api/v2 prefix |
| Global --json/--output flags captured as trailing args | Added run_json_with_body/run_with_body helpers | CRUD operations couldn't send request bodies |

### Bugs Documented (Deferred)

| Bug | Status |
|-----|--------|
| +search helper uses deprecated Jira search API (HTTP 410) | Needs updating to enhanced search |
| +create helper: --project global flag not forwarded to helper parser | Clap trailing_var_arg conflict |

### Key Decisions

| Decision | Phase | Rationale |
|----------|-------|-----------|
| Three-tier spec loading (cache → network → bundled) | 9 | Graceful degradation when network unavailable |
| Confluence v2 API spec (openapi-v2.v3.json) | 11 | v1 spec missing page CRUD, v2 has 212 operations |
| Credential site + spec path prefix for URL resolution | 10 | Supports APIs with path prefixes (e.g., /wiki/api/v2) |
| ADF format required for Jira comment/worklog bodies | 10 | Jira Cloud rejects plain text in v3 API |

## v0.3 Test Coverage & Entity Expansion

**Completed:** 2026-03-23
**Duration:** 1 day
**Version:** 0.3.0

### Stats

| Metric | Value |
|--------|-------|
| Phases | 5 |
| Plans | 9 |
| New E2E tests | 31 |
| Total tests | 70 (411 unit + 7 integration + 70 E2E) |
| Clippy warnings fixed | 7 (Phase 13) |
| Bugs fixed | 2 (+search deprecated API, +create flag forwarding) |

### Key Accomplishments

- **Jira Platform: 20 entity groups tested.** Full CRUD for issues, comments, worklogs, filters, dashboards, versions, components, issue types, groups. Lifecycle tests for watchers, votes, issue links. Read-only for projects, statuses, priorities, resolutions, fields. Attachment test with multipart fallback.
- **Jira Software: 6 tests covering all non-DevOps entity groups.** Board CRUD (with filter dependency), Sprint lifecycle, Epic operations, JSW issue get, board list, backlog move with graceful handling.
- **Confluence: 20 entity groups tested.** Page CRUD, blog post CRUD, space properties CRUD, folder CRUD, whiteboard CRUD, content properties CRUD. Read-only for comments, versions, likes, attachments, tasks, labels, custom content, ancestors, descendants, space roles.
- **Feature gap tests.** Pagination --limit verification, verbose (-v) and trace (--trace) logging output, ADF comment round-trip (create → read → verify content).
- **Code quality.** 23 new unit tests (Phase 13), 7 clippy warnings eliminated, +search updated to enhanced search API, global shorthand flags forwarded to helpers.

### Key Decisions

| Decision | Phase | Rationale |
|----------|-------|-----------|
| +search uses search-and-reconsile-issues-using-jql | 13 | Old endpoint returned HTTP 410 |
| Global shorthand flags forwarded to helpers | 13 | --project, --assignee, --status reach +create |
| Attachment test falls back to read-only | 14 | CLI executor sends JSON, not multipart/form-data |
| Vote test graceful early return on own-issue | 14 | Jira blocks voting on reporter's issues (API constraint) |
| Board creation requires Jira Platform filter | 15 | JSW boards need a filter ID from the Platform API |

---
*Milestones log — Updated on milestone completion*
