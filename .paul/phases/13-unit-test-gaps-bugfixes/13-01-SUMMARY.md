---
phase: 13-unit-test-gaps-bugfixes
plan: 01
subsystem: testing
tags: [unit-tests, clippy, bugfix, search, create, helpers]

provides:
  - 23 new unit tests (388→411)
  - Zero clippy warnings
  - +search uses new enhanced search API
  - +create receives --project from global flag
affects: [14-jira-platform-top20, 15-jira-software-full, 16-confluence-top20]

key-files:
  modified: [src/auth/credentials.rs, src/config.rs, src/helpers.rs, src/main.rs, src/spec/model.rs, src/cli.rs, src/cmd/tree.rs, src/logging.rs, tests/fixtures/jira_test_spec.json, tests/e2e/confluence.rs]

duration: ~15min
completed: 2026-03-23
---

# Phase 13 Plan 01: Unit Test Gaps + Bug Fixes + Clippy Summary

**23 new unit tests, 7 clippy fixes, 2 bug fixes (+search, +create), zero warnings**

## Results

| Metric | Before | After |
|--------|--------|-------|
| Unit tests | 388 | 411 (+23) |
| Clippy warnings | 7 | 0 |
| Known bugs | 2 | 0 |

## Bug Fixes

1. **+search deprecated API:** Changed operation ID from `searchForIssuesUsingJql` to `search-and-reconsile-issues-using-jql`
2. **+create --project routing:** Global shorthand flags (--project, --assignee, --status) now forwarded to helper args before dispatch

## New Unit Tests

| Module | Before | After | Tests Added |
|--------|--------|-------|-------------|
| spec/model.rs | 0 | 8 | HttpMethod Display, ParameterLocation, ApiSpec roundtrip, Operation fields |
| cli.rs | 0 | 8 | Output/Color variants, version/help flags, global flags, subcommand parsing |
| cmd/tree.rs | 6 | 10 | Empty spec, empty tag, no params, no summary |
| logging.rs | 3 | 6 | Auto without NO_COLOR, Always ignores NO_COLOR, Never ignores NO_COLOR |
| config.rs | - | - | Fixed field_reassign_with_default (no new tests, existing tests pass) |

---
*Phase: 13-unit-test-gaps-bugfixes, Plan: 01*
*Completed: 2026-03-23*
