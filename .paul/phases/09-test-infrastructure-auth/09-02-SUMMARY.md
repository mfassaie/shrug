---
phase: 09-test-infrastructure-auth
plan: 02
subsystem: infra
tags: [spec-fetching, cache-refresh, reqwest, atlassian-cdn, openapi, swagger]

requires:
  - phase: 09-test-infrastructure-auth
    provides: E2E test harness (09-01), blocked on empty bundled specs
provides:
  - Network spec fetching in SpecLoader (cache → network → bundled)
  - `shrug cache refresh` command (all or single product)
  - 1,015 real API operations across 5 products
  - E2E smoke tests validated against live Atlassian Cloud
affects: [09-03-auth-tests, 10-jira-crud-tests, 11-confluence-crud-tests, 12-cli-feature-tests]

tech-stack:
  added: []
  patterns: [three-tier spec loading (cache → network → bundled), eprintln progress for user feedback during downloads]

key-files:
  created: []
  modified: [src/spec/registry.rs, src/cli.rs, src/main.rs, tests/e2e/harness.rs, tests/e2e/smoke.rs]

key-decisions:
  - "fetch_spec is private, progress output only in public refresh/refresh_all methods"
  - "HTTP status validation before parsing — non-200 returns ShrugError with status and URL"
  - "Real Jira search operation is search-and-reconsile-issues-using-jql under 'Issue search' tag (old searchForIssuesUsingJql removed by Atlassian)"
  - "run_json() prepends --output json before subcommand due to clap trailing_var_arg parsing"

patterns-established:
  - "Three-tier spec loading: cache (TTL) → network fetch → bundled fallback"
  - "E2E smoke test pattern: create temp profile → run operation → clean up profile"

duration: ~15min
completed: 2026-03-23
---

# Phase 9 Plan 02: Spec Fetching & Cache Refresh Summary

**Network spec fetching from Atlassian CDN, cache refresh command, and E2E smoke test validation against live Cloud (1,015 operations across 5 products)**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~15 min |
| Completed | 2026-03-23 |
| Tasks | 2 completed |
| Files modified | 5 |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: Network fetch in SpecLoader | Pass | Three-tier loading: cache → network → bundled. Network fetches on cache miss. |
| AC-2: Cache refresh command | Pass | `shrug cache refresh` downloads all 5 specs (1,015 operations total) |
| AC-3: Single product refresh | Pass | `shrug cache refresh --product jira` downloads only Jira (620 operations) |
| AC-4: Graceful fallback | Pass | Network failure logs warning, falls back to bundled spec |
| AC-5: E2E smoke test passes | Pass | All 4 tests pass against live Atlassian Cloud (0.76s) |

## Accomplishments

- Added `fetch_spec()` to SpecLoader: downloads spec from Atlassian CDN via reqwest::blocking, validates HTTP status (non-200 returns clear error), parses with format auto-detection (V3/V2), and saves to cache. Progress output via `eprintln!` in public `refresh()`/`refresh_all()` methods only.
- Updated `load()` method with three-tier strategy: cache (TTL-based) → network fetch → bundled fallback. Network failures are non-fatal, logged as warnings.
- Added `CacheCommands::Refresh` subcommand to CLI with optional `--product` flag. `handle_cache()` in main.rs resolves product names via `Product::from_cli_prefix()` and prints operation counts per product.
- Fixed `run_json()` in E2E harness to prepend `--output json` before the subcommand (clap's `trailing_var_arg` captures it as a dynamic parameter otherwise).
- Corrected smoke test to use `search-and-reconsile-issues-using-jql` under "Issue search" tag with bounded JQL query. The old `searchForIssuesUsingJql` was removed by Atlassian (HTTP 410).

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `src/spec/registry.rs` | Modified | fetch_spec(), refresh(), refresh_all(), load() network tier |
| `src/cli.rs` | Modified | CacheCommands enum with Refresh subcommand |
| `src/main.rs` | Modified | handle_cache() function, CacheCommands import |
| `tests/e2e/harness.rs` | Modified | Fixed run_json() argument ordering |
| `tests/e2e/smoke.rs` | Modified | Corrected search operation name and added bounded JQL |

## Decisions Made

| Decision | Rationale | Impact |
|----------|-----------|--------|
| fetch_spec() is private, progress in refresh() only | Silent network fetch during load() avoids surprising output. Users see progress only when explicitly refreshing. | Clean UX for normal operation |
| Bounded JQL required for enhanced search | Atlassian's new search endpoint rejects unbounded queries (security measure). E2E tests must use project-scoped JQL. | All future search tests need project key |
| Temp profile creation in smoke test | CLI requires a profile before env var auth works. Smoke test creates/deletes a temp profile. | Pattern for all E2E tests needing auth |

## Deviations from Plan

### Summary

| Type | Count | Impact |
|------|-------|--------|
| Auto-fixed | 2 | Essential fixes for real API compatibility |
| Scope additions | 0 | None |
| Deferred | 0 | None |

**Total impact:** Essential fixes, no scope creep.

### Auto-fixed Issues

**1. run_json() argument ordering**
- **Found during:** Task 2 (E2E validation)
- **Issue:** `--output json` appended after subcommand was captured as a trailing arg
- **Fix:** Prepend `--output json` before the subcommand args
- **Files:** tests/e2e/harness.rs

**2. Search operation name and JQL**
- **Found during:** Task 2 (E2E validation)
- **Issue:** `searchForIssuesUsingJql` removed by Atlassian (HTTP 410). New endpoint requires bounded JQL.
- **Fix:** Changed to `search-and-reconsile-issues-using-jql` with project-scoped JQL
- **Files:** tests/e2e/smoke.rs

## Issues Encountered

| Issue | Resolution |
|-------|------------|
| Unit test `loader_falls_back_to_bundled_on_cache_miss` failed | Test now succeeds with network fetch. Renamed to `loader_loads_spec_on_cache_miss` with flexible assertion. |
| Deprecated Jira search endpoint removed (HTTP 410) | Switched to enhanced search endpoint |
| Enhanced search requires bounded JQL | Added project-scoped JQL from E2E config |

## Next Phase Readiness

**Ready:**
- All 5 product specs cached with real operations (1,015 total)
- E2E smoke tests passing against live Atlassian Cloud
- Spec fetching unblocks all future E2E phases

**Concerns:**
- BitBucket spec has many operations without operationId (logged as WARN, skipped). 100 operations parsed out of ~335 documented.
- Temp profile pattern needs refinement — could be part of harness setup rather than per-test.

**Blockers:**
- None for Plan 09-03 (auth workflow tests)

---
*Phase: 09-test-infrastructure-auth, Plan: 02*
*Completed: 2026-03-23*
