---
phase: 10-jira-crud-tests
plan: 01
subsystem: testing
tags: [e2e, jira, crud, issues, comments, worklogs, live-cloud]

requires:
  - phase: 09-test-infrastructure-auth
    provides: E2E harness, spec fetching, auth workflow tests
provides:
  - Issue CRUD lifecycle test (create → read → update → delete)
  - Comment CRUD lifecycle test
  - Worklog CRUD lifecycle test
  - run_json_with_body() and run_with_body() harness helpers
  - URL resolution bugfix (credential site over spec placeholder)
affects: [10-02-supporting-crud, 10-03-project-read-entities]

tech-stack:
  added: []
  patterns: [CRUD lifecycle test pattern, ADF body format for comments/worklogs]

key-files:
  created: [tests/e2e/jira.rs]
  modified: [tests/e2e/main.rs, tests/e2e/harness.rs, src/executor.rs]

key-decisions:
  - "Fixed resolve_base_url() to prefer credential site over spec server URL — spec URLs are placeholders"
  - "Added run_json_with_body() / run_with_body() to ShrugRunner for global --json flag prepending"
  - "Parameter format uses separate flag and value (--param value), not equals (--param=value)"
  - "ADF format required for comment and worklog bodies in Jira Cloud"

patterns-established:
  - "CRUD test pattern: setup_profile → create resource → read → update → read-verify → delete → teardown"
  - "Global flags (--json, --output) must be prepended before subcommand due to trailing_var_arg"

duration: ~20min
completed: 2026-03-23
---

# Phase 10 Plan 01: Core Jira CRUD Tests Summary

**3 full CRUD lifecycle tests (Issues, Comments, Worklogs) passing against live Atlassian Cloud with URL resolution bugfix**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~20 min |
| Completed | 2026-03-23 |
| Tasks | 2 completed |
| Files created | 1 |
| Files modified | 3 |
| New E2E tests | 3 (16 total) |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: Issue CRUD | Pass | Created TEST-3, read/verified summary, updated, verified update, deleted |
| AC-2: Comment CRUD | Pass | Added ADF comment, read, updated, deleted on parent issue |
| AC-3: Worklog CRUD | Pass | Added 1h worklog, read, updated to 2h, deleted on parent issue |

## Accomplishments

- Created `tests/e2e/jira.rs` with 3 CRUD lifecycle tests covering Issues (create/read/update/delete), Comments (add/read/update/delete with ADF bodies), and Worklogs (add/read/update/delete with time tracking).
- Fixed critical URL resolution bug in `src/executor.rs`: `resolve_base_url()` was using the spec's placeholder URL (`your-domain.atlassian.net`) instead of the user's actual site. Changed to prefer credential site URL, with spec URL as fallback.
- Added `run_json_with_body()` and `run_with_body()` helper methods to ShrugRunner for operations that require a JSON request body (global `--json` flag must be prepended before the subcommand).

## Deviations from Plan

| Type | Count | Impact |
|------|-------|--------|
| Auto-fixed | 2 | Critical bug fixes discovered by E2E testing |

**1. URL resolution bug** — `resolve_base_url()` prioritised spec server URL over credential site. All API calls went to `your-domain.atlassian.net` instead of the user's site. Fixed: credential site now takes priority.

**2. Parameter format** — `--param=value` format not supported by dynamic command parser. Changed to `--param value` (separate args).

## Next Phase Readiness

**Ready:** 16 E2E tests passing, core CRUD pattern established
**Concerns:** None
**Blockers:** None

---
*Phase: 10-jira-crud-tests, Plan: 01*
*Completed: 2026-03-23*
