---
phase: 09-test-infrastructure-auth
plan: 01
subsystem: testing
tags: [e2e, assert_cmd, atlassian-cloud, integration-tests, ci]

requires:
  - phase: 08-distribution-polish
    provides: Built binary, CI workflows, all CLI features to test
provides:
  - E2E test harness with CLI runner and env config
  - Skip guard macro for graceful test skipping
  - ResourceTracker with Drop-based panic-safe cleanup
  - 4 smoke tests (version, help, spec loading, live API)
  - Updated E2E CI workflow
affects: [09-02-auth-tests, 10-jira-crud-tests, 11-confluence-crud-tests, 12-cli-feature-tests]

tech-stack:
  added: []
  patterns: [skip_unless_e2e!() macro for conditional E2E tests, ShrugRunner wrapping assert_cmd, ResourceTracker with Drop cleanup]

key-files:
  created: [tests/e2e/main.rs, tests/e2e/harness.rs, tests/e2e/fixtures.rs, tests/e2e/smoke.rs]
  modified: [.github/workflows/e2e.yml]

key-decisions:
  - "Multi-file integration test binary (tests/e2e/main.rs + modules) for maintainability across 4 phases"
  - "ResourceTracker uses drain() in cleanup + Drop trait for panic-safe resource lifecycle"
  - "ShrugRunner sets SHRUG_SITE/EMAIL/API_TOKEN env vars, mapping from E2E-prefixed vars"
  - "30s default command timeout via assert_cmd (SHRUG_E2E_TIMEOUT_SECS configurable)"

patterns-established:
  - "E2E test pattern: skip_unless_e2e!() → ShrugRunner::new() → run/run_json → assert_success()"
  - "Fixture pattern: ResourceTracker::new() → create helpers → automatic Drop cleanup"

duration: ~10min
completed: 2026-03-23
---

# Phase 9 Plan 01: E2E Test Harness & Fixture Framework Summary

**E2E test harness with CLI runner, env config, skip guard, panic-safe fixture cleanup, 4 smoke tests, and updated CI workflow**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~10 min |
| Completed | 2026-03-23 |
| Tasks | 2 completed |
| Files created | 4 |
| Files modified | 1 |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: Environment Configuration | Pass | E2eConfig reads SHRUG_E2E_SITE, EMAIL, TOKEN from env vars |
| AC-2: Skip Guard | Pass | skip_unless_e2e!() macro returns early, 4 tests skip in 0.00s |
| AC-3: CLI Runner | Pass | ShrugRunner wraps assert_cmd::cargo_bin, captures stdout/stderr/exit_code |
| AC-4: Resource Tracking and Teardown | Pass | ResourceTracker with Drop impl, reverse-order cleanup, drain() for idempotency |
| AC-5: Rate Limit Awareness | Pass | rate_limit_delay() with 200ms default, configurable via SHRUG_E2E_DELAY_MS |
| AC-6: Smoke Test Validation | Pass (structural) | Harness compiles and tests skip gracefully. Live validation pending user credentials |

## Accomplishments

- Created `tests/e2e/` multi-file integration test binary with 4 modules (main, harness, fixtures, smoke). Completely isolated from existing mock-based `tests/integration.rs`.
- `ShrugRunner` wraps `assert_cmd::Command::cargo_bin("shrug")` with automatic credential injection, 30s configurable timeout, and `run_json()` for JSON response parsing. `RunResult` includes `assert_success()`, `assert_exit_code()`, `assert_stdout_contains()`, and `json_field()` helpers.
- `ResourceTracker` implements `Drop` for panic-safe cleanup. Uses `drain()` so cleanup is idempotent (safe to call manually + via Drop). Reverse-order iteration handles dependent resources.
- Updated `.github/workflows/e2e.yml` with `workflow_dispatch` inputs for site URL, Jira project, and Confluence space. Runs `cargo test --test e2e --release -- --test-threads=1` for sequential, rate-limited execution.

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `tests/e2e/main.rs` | Created | E2E test binary entry point with module declarations |
| `tests/e2e/harness.rs` | Created | E2eConfig, skip_unless_e2e!(), ShrugRunner, RunResult with assertion helpers |
| `tests/e2e/fixtures.rs` | Created | ResourceTracker with Drop, create_test_issue/delete_test_issue helpers |
| `tests/e2e/smoke.rs` | Created | 4 smoke tests: version, help, jira spec loading, live API connection |
| `.github/workflows/e2e.yml` | Modified | Replaced individual smoke steps with comprehensive cargo test runner |

## Decisions Made

| Decision | Rationale | Impact |
|----------|-----------|--------|
| Multi-file test binary (main.rs + modules) | Phases 10-12 will add auth, jira, confluence, feature test modules | Clean organisation as test suite grows |
| ResourceTracker drain() + Drop | Manual cleanup + automatic panic-safe cleanup without double-execution | Prevents orphaned test data in shared Cloud |
| 200ms default inter-request delay | Conservative for Atlassian burst rate limits (100 GET/POST per second) | Configurable via env var if too slow |

## Deviations from Plan

### Summary

| Type | Count | Impact |
|------|-------|--------|
| Auto-fixed | 0 | None |
| Scope additions | 0 | None |
| Deferred | 0 | None |

**Total impact:** Plan executed as written (with audit amendments applied).

## Issues Encountered

| Issue | Resolution |
|-------|------------|
| Pre-existing clippy warnings in src/ (6 warnings in credentials.rs, config.rs) | Outside plan scope (src/** boundary). E2E test code is clippy-clean. |
| dead_code warnings for future-use items (fixtures, assertion helpers) | Added #[allow(dead_code)] since these are consumed by Phases 10-12 |

## Next Phase Readiness

**Ready:**
- E2E harness compiles and tests skip correctly without credentials
- Fixture framework ready for CRUD test helpers in Phases 10-11
- CI workflow ready for live execution
- Pattern established for all future E2E tests

**Concerns:**
- Live API smoke tests not yet validated (user credentials needed)
- Pre-existing clippy warnings in src/ should be addressed before next feature milestone

**Blockers:**
- None for Plan 09-02 (auth workflow tests)

---
*Phase: 09-test-infrastructure-auth, Plan: 01*
*Completed: 2026-03-23*
