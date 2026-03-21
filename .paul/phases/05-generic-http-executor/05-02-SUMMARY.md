---
phase: 05-generic-http-executor
plan: 02
subsystem: api
tags: [retry, backoff, jitter, rate-limit, retry-after]

requires:
  - phase: 05-generic-http-executor
    provides: execute() function, send_request helper, ShrugError variants
provides:
  - Transparent retry logic for 429/5xx and transient network errors
  - Exponential backoff with jitter (1s, 2s, 4s, 8s + 0-50% jitter)
  - Retry-After header parsing (integer seconds, capped at 60s)
  - Network error classification (timeout/connect retryable, DNS not)
affects: [05-03-pagination, 05-04-quirks]

tech-stack:
  added: []
  patterns: [SendResult enum for retry classification, backoff with jitter]

key-files:
  created: []
  modified: [src/executor.rs, src/auth/credentials.rs, src/config.rs]

key-decisions:
  - "SendResult enum (Success/Retryable/Fatal) for clean retry flow control"
  - "Network errors classified via reqwest::Error methods (is_timeout, is_connect)"
  - "Debug-level logging of intermediate retry response bodies"

patterns-established:
  - "Retry classification: retryable (429, 5xx, timeout, connect) vs fatal (4xx, DNS)"
  - "Backoff formula: base * 2^attempt + jitter(0..50%)"

duration: ~10min
completed: 2026-03-21
---

# Phase 5 Plan 02: Rate Limiting, Retries, and Error Handling Summary

**Retry wrapper with exponential backoff, jitter, Retry-After parsing, and transient network error retries around the HTTP executor.**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~10min |
| Completed | 2026-03-21 |
| Tasks | 2 completed |
| Files modified | 3 |
| Tests added | 22 (247 total) |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: Retry on 429 with Retry-After | Pass | Parses header, caps at 60s, retries with parsed delay |
| AC-2: Retry on 5xx with exponential backoff | Pass | 1s/2s/4s/8s base + 0-50% jitter |
| AC-3: Maximum retry limit | Pass | 5 total attempts (1 initial + 4 retries) |
| AC-4: Non-retryable errors pass through | Pass | 400/401/403/404 return immediately |
| AC-5: Retry-After header parsing | Pass | Integer seconds parsed, non-numeric falls back to backoff |
| AC-6: Dry-run skips retry logic | Pass | Dry-run returns before retry loop |
| AC-7: Retry on transient network errors | Pass | Timeout/connect retried, DNS/request not |

## Accomplishments

- Refactored execute() into send_request() helper + retry loop wrapper
- Added SendResult enum (Success/Retryable/Fatal) for clean flow control
- Network error classification using reqwest::Error methods
- Fixed flaky test races in credentials.rs and config.rs (env var + cwd mutex guards)

## Task Commits

| Task | Commit | Type | Description |
|------|--------|------|-------------|
| Tasks 1-2 | `1f40a06` | feat | Retry wrapper, backoff, tests, flaky test fixes |

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `src/executor.rs` | Modified | Retry wrapper, SendResult enum, backoff calculation, network error classification |
| `src/auth/credentials.rs` | Modified | Env var test mutex (flaky test fix) |
| `src/config.rs` | Modified | Cwd test mutex (flaky test fix) |

## Decisions Made

| Decision | Rationale | Impact |
|----------|-----------|--------|
| SendResult enum for retry classification | Cleaner than nested match/if chains. Each send attempt returns Success, Retryable, or Fatal. | Retry loop is readable and extensible |
| Retry-After cap at 60s | Prevents server from stalling CLI indefinitely | Safety bound for batch workflows |
| Debug-level logging of intermediate bodies | Aids diagnosis of intermittent failures without verbose output | Accessible via -vv flag |

## Deviations from Plan

### Summary

| Type | Count | Impact |
|------|-------|--------|
| Auto-fixed | 2 | Flaky test races in credentials.rs and config.rs |
| Scope additions | 0 | None |
| Deferred | 0 | None |

**Total impact:** Flaky test fixes were essential for CI stability. No scope creep.

## Issues Encountered

| Issue | Resolution |
|-------|------------|
| Flaky test `resolve_env_vars_override` in credentials.rs | Added ENV_LOCK mutex, same pattern as config.rs |
| Flaky test `project_config_search_stops_at_git_root` in config.rs | Added ENV_LOCK mutex for cwd-mutating tests |

## Next Phase Readiness

**Ready:**
- Retry logic is transparent to callers (execute() signature unchanged)
- Pagination (05-03) can call execute() in a loop and retries happen per-page
- Quirks (05-04) can add pre/post-request hooks without affecting retry logic

**Concerns:**
- None

**Blockers:**
- None

---
*Phase: 05-generic-http-executor, Plan: 02*
*Completed: 2026-03-21*
