# Roadmap: shrug

## Overview

A dynamic CLI for Atlassian Cloud — commands generated at runtime from OpenAPI specs, supporting Jira, Jira Software, and Confluence. Built in Rust, inspired by Google Workspace CLI architecture.

## Current Milestone

**v0.3 Test Coverage & Entity Expansion** (v0.3.0)
Status: In Progress
Phases: 4 of 5 complete

## Phases

| Phase | Name | Plans | Status | Completed |
|-------|------|-------|--------|-----------|
| 13 | Unit Test Gaps + Bug Fixes + Clippy | 1 | Complete | 2026-03-23 |
| 14 | Jira Platform Top 20 | 2 | Complete | 2026-03-23 |
| 15 | Jira Software Full Coverage | 2 | Complete | 2026-03-23 |
| 16 | Confluence Top 20 | 3 | Complete | 2026-03-23 |
| 17 | E2E Feature Gaps | TBD | Not started | - |

## Phase Details

### Phase 13: Unit Test Gaps + Bug Fixes + Clippy

**Focus:** Fill unit test gaps, fix known bugs, eliminate clippy warnings
**Plans:** TBD

**Scope:**
- Unit tests for cli.rs (0→10+), spec/model.rs (0→10+)
- Strengthen completions.rs, cmd/tree.rs, logging.rs
- Fix +search deprecated API (HTTP 410)
- Fix +create --project flag routing
- Fix 6 clippy warnings in credentials.rs, config.rs

### Phase 14: Jira Platform Top 20

**Focus:** Expand Jira Platform E2E from 14 to 20 entities
**Depends on:** Phase 13 (bug fixes)
**Plans:** TBD

**Scope:**
- Issue attachments (add, get, delete)
- Issue links (create, get, delete)
- Issue watchers (add, get, remove)
- Issue votes (add, get, remove)
- Groups (create, get, delete)
- Issue types (create, get, update, delete)

### Phase 15: Jira Software Full Coverage

**Focus:** E2E tests for all Jira Software entity groups
**Depends on:** Phase 13 (bug fixes)
**Plans:** TBD

**Scope:**
- Board (create, get, list, config, delete)
- Sprint (create, get, update, get issues, move issues)
- Epic (get, get issues, move issues)
- Issue (rank, estimation)
- Backlog (move to backlog)
- Note: DevOps entities (builds, deployments, feature flags) excluded — require Forge/Connect permissions

### Phase 16: Confluence Top 20

**Focus:** Expand Confluence E2E from 5 to 20 entities
**Depends on:** Phase 13 (bug fixes)
**Plans:** TBD

**Scope:**
- Blog Post CRUD, Comment CRUD, Attachment, Content Properties CRUD
- Custom Content CRUD, Version, Like, Children, Ancestors, Descendants
- Space Properties CRUD, Folder, Task, Space Roles, Whiteboard

### Phase 17: E2E Feature Gaps

**Focus:** Test CLI features not yet covered by E2E
**Depends on:** Phases 14-16 (entity tests provide data context)
**Plans:** TBD

**Scope:**
- Pagination: --page-all with multiple pages, --limit
- ADF rendering: Markdown → ADF input, ADF → terminal output
- Verbose logging: --trace request/response, -v/-vv levels

## Constraints

- BitBucket and JSM excluded from testing and roadmap
- Jira Software has ~5 entity groups (not 20) — test all available
- Live Atlassian Cloud credentials required
- Tests must clean up, sequential execution

## Completed Milestones

<details>
<summary>v0.2 E2E Validation - 2026-03-23 (4 phases, 8 plans)</summary>

| Phase | Name | Plans | Completed |
|-------|------|-------|-----------|
| 9 | Test Infrastructure & Auth | 3 | 2026-03-23 |
| 10 | Jira CRUD Tests | 2 | 2026-03-23 |
| 11 | Confluence CRUD Tests | 1 | 2026-03-23 |
| 12 | CLI Feature Tests | 1 | 2026-03-23 |

Full archive: `.paul/milestones/v0.2.0-ROADMAP.md`

</details>

<details>
<summary>v0.1 MVP - 2026-03-21 (8 phases, 24 plans)</summary>

| Phase | Name | Plans | Completed |
|-------|------|-------|-----------|
| 1 | Project Foundation | 3 | 2026-03-21 |
| 2 | OpenAPI Spec Engine | 4 | 2026-03-21 |
| 3 | Dynamic Command Tree | 2 | 2026-03-21 |
| 4 | Authentication & Profiles | 3 | 2026-03-21 |
| 5 | Generic HTTP Executor | 4 | 2026-03-21 |
| 6 | Output & Formatting | 2 | 2026-03-21 |
| 7 | Helper Commands & ADF | 3 | 2026-03-21 |
| 8 | Distribution & Polish | 3 | 2026-03-21 |

Full archive: `.paul/milestones/v0.1.0-ROADMAP.md`

</details>

---
*Roadmap created: 2026-03-21*
*Last updated: 2026-03-23 after v0.3 milestone creation*
