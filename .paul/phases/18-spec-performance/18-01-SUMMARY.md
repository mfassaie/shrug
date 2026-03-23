---
phase: 18-spec-performance
plan: 01
subsystem: spec
tags: [rkyv, binary-cache, zero-copy, serialisation, performance]

requires:
  - phase: 02-spec-loading
    provides: JSON spec caching with TTL
provides:
  - rkyv binary spec cache with dual-write and JSON fallback
  - zero-copy deserialisation path for warm starts
affects: [18-02-etag-refresh]

tech-stack:
  added: [rkyv 0.8.15 with bytecheck]
  patterns: [dual-write cache (binary + JSON), binary-preferred load with JSON fallback]

key-files:
  created: []
  modified: [Cargo.toml, src/spec/model.rs, src/spec/cache.rs]

key-decisions:
  - "Binary logic in cache.rs not registry.rs: SpecCache handles binary internally, SpecLoader gets it transparently"
  - "rkyv 0.8.15 with bytecheck feature (not validation — renamed in 0.8)"
  - "Atomic write pattern: write to .rkyv.tmp.{pid} then rename"

patterns-established:
  - "Dual-write cache: save() writes both JSON (metadata/TTL) and binary (fast load)"
  - "Binary-preferred load: load() tries binary first, falls back to JSON"

duration: ~45min
started: 2026-03-23T00:00:00Z
completed: 2026-03-23T00:00:00Z
---

# Phase 18 Plan 01: rkyv Binary Spec Cache Summary

**rkyv zero-copy binary spec cache with dual-write and JSON fallback for fast warm starts.**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~45min |
| Tasks | 3 planned, 2 executed (Task 3 unnecessary) |
| Files modified | 3 (Cargo.toml, model.rs, cache.rs) |
| Tests added | 7 new binary cache unit tests |
| Total tests | 495 (418 unit + 70 E2E + 7 integration), 1 ignored |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: rkyv derives compile on all model types | Pass | All 7 types: ApiSpec, Tag, Operation, HttpMethod, Parameter, ParameterLocation, RequestBody |
| AC-2: Binary cache round-trip | Pass | save_binary → load_binary verified in unit tests |
| AC-3: Fallback to JSON on binary miss | Pass | load() falls back to JSON when no .rkyv file exists |

## Accomplishments

- rkyv Archive/Serialize/Deserialize derives on all 7 spec model types
- Binary save/load/invalidate methods on SpecCache with atomic write pattern
- Dual-write save (JSON + binary), binary-preferred load with JSON fallback
- 7 new unit tests covering round-trip, fallback, corruption recovery, dual-write, invalidation

## Task Commits

All changes in a single apply commit (WIP pattern for mid-phase work):

| Task | Commit | Type | Description |
|------|--------|------|-------------|
| Task 1: rkyv derives | `f3e6086` | feat | Added rkyv 0.8.15 dep, derives on 7 model types |
| Task 2: Binary cache methods | `f3e6086` | feat | save_binary, load_binary, invalidate_binary, dual-write in save/load |
| Task 3: SpecLoader update | skipped | — | Not needed, binary logic handled in cache.rs |

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `Cargo.toml` | Modified | Added rkyv 0.8.15 with bytecheck feature |
| `src/spec/model.rs` | Modified | Added rkyv Archive/Serialize/Deserialize derives on 7 types |
| `src/spec/cache.rs` | Modified | Added save_binary, load_binary, invalidate_binary, updated save/load/load_stale/invalidate for dual-write and binary-preferred load |

## Decisions Made

| Decision | Rationale | Impact |
|----------|-----------|--------|
| Binary logic in cache.rs, not registry.rs | SpecCache already owns load/save. Binary preference is a cache concern, not a loader concern. Keeps SpecLoader unchanged | Task 3 from plan was unnecessary |
| rkyv 0.8.15 with bytecheck (not validation) | Feature was renamed in rkyv 0.8 | Must use bytecheck, not validation |
| Non-fatal binary write failures | Binary cache is an optimisation. If binary write fails, JSON still works | Prevents binary issues from blocking spec saves |

## Deviations from Plan

### Summary

| Type | Count | Impact |
|------|-------|--------|
| Scope reduction | 1 | Task 3 unnecessary — positive simplification |
| Auto-fixed | 0 | — |
| Deferred | 0 | — |

**Total impact:** Simpler than planned. Binary logic at cache layer means fewer touch points.

### Details

**1. Task 3 (SpecLoader update) skipped**
- **Planned:** Update `registry.rs` SpecLoader to explicitly use binary cache
- **Actual:** Not needed. `SpecCache::load()` and `load_stale()` already handle binary-preferred loading internally. SpecLoader delegates to SpecCache and gets binary performance transparently
- **Impact:** Positive. One fewer file to modify, cleaner abstraction boundary

## Issues Encountered

None.

## Next Phase Readiness

**Ready:**
- Binary cache infrastructure complete for Phase 18-02 (ETag refresh)
- Dual-write pattern means background refresh can update both formats atomically

**Concerns:**
- None

**Blockers:**
- None

---
*Phase: 18-spec-performance, Plan: 01*
*Completed: 2026-03-23*
