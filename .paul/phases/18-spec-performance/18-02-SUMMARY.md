---
phase: 18-spec-performance
plan: 02
subsystem: spec
tags: [etag, conditional-fetch, serve-stale, background-refresh, caching]

requires:
  - phase: 18-spec-performance
    provides: rkyv binary cache, dual-write save, binary-preferred load
provides:
  - ETag-based conditional fetch with 304 Not Modified handling
  - Serve-stale pattern with background refresh thread
  - TTL touch without spec re-save
affects: []

tech-stack:
  added: []
  patterns: [serve-stale with background refresh, conditional GET with If-None-Match, fire-and-forget thread]

key-files:
  created: []
  modified: [src/spec/cache.rs, src/spec/registry.rs]

key-decisions:
  - "Background refresh uses std::thread::spawn, not tokio, matching the blocking architecture"
  - "Background thread creates its own SpecCache and reqwest client to avoid Send/Sync issues"
  - "parse_spec auto-detects V2/V3 format, so spec_format param is informational only"

patterns-established:
  - "Serve-stale: expired cache returned immediately, background thread conditionally refreshes"
  - "Conditional fetch: If-None-Match header sent when ETag stored, 304 just touches TTL"
  - "write_json_entry: extracted helper for atomic JSON cache writes (used by save_with_etag and touch_ttl)"

duration: ~20min
started: 2026-03-23T00:00:00Z
completed: 2026-03-23T00:00:00Z
---

# Phase 18 Plan 02: ETag Conditional Fetch + Serve-Stale Background Refresh Summary

**ETag-based conditional fetch with serve-stale pattern: expired specs returned instantly, background thread refreshes cache for next invocation.**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~20min |
| Tasks | 2 planned, 2 executed |
| Files modified | 2 (cache.rs, registry.rs) |
| Tests added | 8 new unit tests |
| Total tests | 503 (426 unit + 70 doc + 7 integration), 1 ignored |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: ETag stored and sent on conditional fetch | Pass | save_with_etag stores ETag, fetch_spec sends If-None-Match |
| AC-2: 304 Not Modified refreshes TTL | Pass | touch_ttl updates cached_at, preserves spec and ETag |
| AC-3: Serve-stale returns immediately with background refresh | Pass | load() serves stale, spawns std::thread for refresh |
| AC-4: No cache falls back to synchronous fetch | Pass | Cold start path unchanged (network then bundled) |

## Accomplishments

- save_with_etag method for storing ETags in CacheMetadata alongside cached specs
- load_etag and touch_ttl methods on SpecCache for conditional fetch support
- Extracted write_json_entry helper (atomic write pattern reused by save and touch_ttl)
- cache_dir() getter for background thread to create its own SpecCache
- fetch_spec now captures ETag response headers and sends If-None-Match conditionals
- 304 Not Modified handling: touches TTL without re-parsing or re-saving spec
- SpecLoader::load() rewritten with serve-stale: fresh → stale+background → sync fetch → bundled
- background_refresh standalone function: fire-and-forget thread with own client and cache

## Task Commits

Not yet committed (WIP, will be committed at phase transition).

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `src/spec/cache.rs` | Modified | Added save_with_etag, load_etag, touch_ttl, write_json_entry, cache_dir; save delegates to save_with_etag |
| `src/spec/registry.rs` | Modified | Rewritten load() for serve-stale, fetch_spec captures ETags, background_refresh function |

## Decisions Made

| Decision | Rationale | Impact |
|----------|-----------|--------|
| std::thread::spawn (not tokio) for background refresh | Codebase is blocking throughout. Adding tokio for one thread would be over-engineering | Simple, no new dependencies |
| Thread creates own SpecCache + reqwest client | SpecCache and reqwest Client aren't Send. Owned instances avoid lifetime issues | Clean thread boundary, ~1ms overhead for creation |
| save() delegates to save_with_etag(None) | Avoids code duplication, backward compatible | Existing callers unchanged |
| write_json_entry extracted as helper | Used by both save_with_etag and touch_ttl for atomic JSON writes | Reduces duplication |

## Deviations from Plan

None. Plan executed exactly as written.

## Issues Encountered

None.

## Next Phase Readiness

**Ready:**
- Phase 18 (Spec Performance) is complete
- rkyv binary cache + ETag conditional fetch + serve-stale all working
- Connection pooling and lazy loading were already implemented (confirmed in handoff)
- Ready for Phase 19 (Confluence Helper) or Phase 20 (Dynamic Completions)

**Concerns:**
- None

**Blockers:**
- None

---
*Phase: 18-spec-performance, Plan: 02*
*Completed: 2026-03-23*
