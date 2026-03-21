---
phase: 08-distribution-polish
plan: 03
subsystem: polish
tags: [error-messages, first-run, e2e-tests, benchmarks, github-actions, workflow-dispatch]

requires:
  - phase: 08-distribution-polish
    provides: Integration test infrastructure (08-02), release pipeline (08-01)
provides:
  - Actionable error remediation hints for all ShrugError variants
  - First-run detection for unconfigured users
  - On-demand E2E smoke test workflow
  - Spec parsing and command tree performance benchmarks
affects: []

tech-stack:
  added: []
  patterns: [remediation hints on error types, first-run detection via profile store, timing-based benchmark tests]

key-files:
  created: [.github/workflows/e2e.yml]
  modified: [src/error.rs, src/main.rs, tests/integration.rs]

key-decisions:
  - "remediation() as separate method, not part of Display — keeps error API stable"
  - "First-run detection uses profile_store.list().is_empty() — audit finding, distinguishes empty store from creds unavailable"
  - "eprintln! for benchmark output — visible without --nocapture, audit finding"

patterns-established:
  - "Error remediation pattern: ShrugError.remediation() returns static hint string"
  - "Benchmark tests as standard #[test] with Instant timing, no nightly features"

duration: ~5min
completed: 2026-03-21
---

# Phase 8 Plan 03: E2E Smoke Tests, Performance Benchmarks, and First-Run Polish Summary

**Actionable error remediation hints for all error variants, first-run detection for unconfigured users, on-demand E2E workflow, and spec parsing/command tree benchmarks**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~5 min |
| Completed | 2026-03-21 |
| Tasks | 2 completed |
| Files modified | 4 |
| New tests | 4 (2 remediation + 2 benchmarks) |
| Total tests | 395 (388 unit + 7 integration, 1 ignored) |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: Actionable error messages | Pass | `remediation()` method returns non-empty hint for all 10 variants |
| AC-2: First-run detection | Pass | `profile_store.list()?.is_empty()` check before product commands |
| AC-3: On-demand E2E smoke test workflow | Pass | `workflow_dispatch` with site URL input, secrets for auth |
| AC-4: Performance benchmarks | Pass | Spec parsing 191µs avg, command tree 7µs avg (100 iterations each) |

## Accomplishments

- Added `ShrugError::remediation()` method returning a static remediation hint for each error variant. Main error handler now prints `Hint: ...` below every error message, giving users concrete next steps.
- First-run detection in the product command handler checks `profile_store.list()?.is_empty()` and returns a clear `AuthError` suggesting `shrug auth setup` when no profiles exist at all. Does not fire when profiles exist but credentials are temporarily unavailable (audit fix).
- Created `.github/workflows/e2e.yml` with `workflow_dispatch` trigger, `site_url` input, secrets-based auth, cargo caching, and 4 smoke test steps (version, help, jira help, live API). Job has `timeout-minutes: 10` (audit fix).
- Added 2 benchmark tests using `std::time::Instant` with `eprintln!` output (audit fix). Both well within generous bounds: spec parsing 19ms/100 iterations, command tree 0.7ms/100 iterations.

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `src/error.rs` | Modified | Added `remediation()` method + 2 tests |
| `src/main.rs` | Modified | Error hint printing + first-run detection |
| `.github/workflows/e2e.yml` | Created | On-demand E2E smoke test workflow |
| `tests/integration.rs` | Modified | 2 benchmark tests (spec parsing, command tree) |

## Decisions Made

| Decision | Rationale | Impact |
|----------|-----------|--------|
| remediation() returns &'static str | No allocations, all hints are compile-time constants | Clean, zero-cost API |
| First-run checks list().is_empty() not profile.is_none() | Audit finding: profile.is_none() conflates empty store with creds unavailable | Correct first-run detection |
| E2E live API step uses `|| true` | Live API may fail due to permissions, step documents intent | Non-blocking smoke test |

## Deviations from Plan

None. Plan executed exactly as written (with audit amendments applied).

## Issues Encountered

None.

## Next Phase Readiness

**Ready:**
- Phase 8 complete (3/3 plans done)
- All 8 phases of v0.1 MVP milestone complete
- 395 tests passing, all audited
- Release pipeline, integration tests, E2E workflow, benchmarks all in place

**Concerns:**
- Performance benchmarks use small test fixture, not full 2.47MB Jira spec
- E2E workflow requires manual secret configuration in repo settings

**Blockers:**
- None

---
*Phase: 08-distribution-polish, Plan: 03*
*Completed: 2026-03-21*
