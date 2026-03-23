---
phase: 15-jira-software-full
plan: 02
subsystem: testing
tags: [jira-software, e2e, epics, backlog, issues]
provides:
  - E2E tests for Epic operations (get, get-issues)
  - E2E test for JSW Issue get
  - E2E test for Backlog move (graceful)
key-files:
  modified: [tests/e2e/jira_software.rs]
duration: ~5min
completed: 2026-03-23T00:00:00Z
---

# Phase 15 Plan 02: Epic + Issue + Backlog E2E Tests

**3 tests completing Jira Software coverage: Epic operations, JSW issue get, and backlog move with graceful handling for issues not on a board.**

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: Epic operations | Pass | Create Epic via Platform API, get-epic + get-issues-for-epic via JSW API |
| AC-2: JSW Issue get | Pass | JSW-specific get-issue endpoint tested |
| AC-3: Backlog move | Pass | Graceful handling if issue not on board |

## Verification Results

- 52 tests pass, zero clippy warnings
- 3 new test functions, 6 total in jira_software.rs

---
*Completed: 2026-03-23*
