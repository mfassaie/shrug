# Roadmap: shrug

## Overview

A dynamic CLI for Atlassian Cloud — commands generated at runtime from OpenAPI specs, supporting Jira, Jira Software, Confluence, BitBucket, and Service Management. Built in Rust, inspired by Google Workspace CLI architecture.

## Current Milestone

**v0.2 E2E Validation** (v0.2.0)
Status: In Progress
Phases: 3 of 4 complete

## Phases

| Phase | Name | Plans | Status | Completed |
|-------|------|-------|--------|-----------|
| 9 | Test Infrastructure & Auth | 3 | Complete | 2026-03-23 |
| 10 | Jira CRUD Tests | 2 | Complete | 2026-03-23 |
| 11 | Confluence CRUD Tests | 1 | Complete | 2026-03-23 |
| 12 | CLI Feature Tests | TBD | Not started | - |

## Phase Details

### Phase 9: Test Infrastructure & Auth

**Focus:** Test harness, live env config, fixtures/teardown framework, all auth workflow tests
**Depends on:** v0.1 MVP (all features built)
**Plans:** TBD (defined during /paul:plan)

**Scope:**
- E2E test harness with live Atlassian Cloud connectivity
- Environment configuration (site URL, credentials, project keys)
- Test fixture framework (setup/teardown, resource cleanup)
- Rate limit awareness in test runner
- Auth workflow tests: API token, OAuth 2.0 + PKCE, token refresh, profile CRUD, encrypted fallback, env var auth, first-run detection

### Phase 10: Jira CRUD Tests

**Focus:** Top 30 Jira entities, full CRUD against live Cloud
**Depends on:** Phase 9 (test infrastructure)
**Plans:** TBD (defined during /paul:plan)

**Scope:**
- Top 30 Jira entities identified from OpenAPI spec
- Full CRUD cycle for each entity (create, read, update, delete)
- Real data assertions (response structure, status codes, field values)
- Teardown ensures no orphaned test data

### Phase 11: Confluence CRUD Tests

**Focus:** Top 30 Confluence entities, full CRUD against live Cloud
**Depends on:** Phase 9 (test infrastructure)
**Plans:** TBD (defined during /paul:plan)

**Scope:**
- Top 30 Confluence entities identified from OpenAPI spec
- Full CRUD cycle for each entity
- Real data assertions
- Teardown ensures no orphaned test data

### Phase 12: CLI Feature Tests

**Focus:** Output formats, pagination, helpers, JQL, ADF, error handling
**Depends on:** Phases 10-11 (entity tests provide data context)
**Plans:** TBD (defined during /paul:plan)

**Scope:**
- Output format tests: table, JSON, YAML, CSV, plain
- Pagination: --page-all, --limit
- --dry-run mode validation
- --fields column selection
- JQL shorthand: --project, --assignee, --status
- Helper commands: +create, +search, +transition
- ADF input (Markdown → ADF) and output (ADF → terminal)
- Error remediation hints
- --verbose / --trace logging

## Constraints

- Requires live Atlassian Cloud credentials
- Tests must clean up after themselves (teardown)
- Rate limit awareness (sequential or throttled, not parallel)
- Tests runnable locally and in CI (secrets-based)
- Test failures produce clear diagnostics (request/response)

## Completed Milestones

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
*Last updated: 2026-03-23 after v0.2 milestone creation*
