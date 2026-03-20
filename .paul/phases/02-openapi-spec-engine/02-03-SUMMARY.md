---
phase: 02-openapi-spec-engine
plan: 03
subsystem: api
tags: [cache, ttl, product-registry, bundled-specs, spec-loader, chrono]

requires:
  - phase: 02-openapi-spec-engine
    provides: ApiSpec (Serialize/Deserialize), parse_spec() unified entry point
  - phase: 01-project-foundation
    provides: ShrugPaths (cache_dir), ShrugConfig (cache_ttl_hours), ShrugError
provides:
  - SpecCache — JSON file cache with TTL, version detection, atomic writes
  - Product enum — 5 Atlassian products with metadata (URLs, formats, cache keys)
  - SpecLoader — tiered loading (cache → bundled → error)
  - Bundled fallback specs for all 5 products
  - Spec version change detection (cached_version, has_version_changed)
affects: [02-04-conformance, 03-command-tree, 05-http-executor]

tech-stack:
  added: [chrono 0.4]
  patterns: [tiered loading, serve-stale cache, atomic file write with PID suffix, cache key validation]

key-files:
  created: [src/spec/cache.rs, src/spec/registry.rs, src/spec/bundled/*.json]
  modified: [src/spec/mod.rs, Cargo.toml]

key-decisions:
  - "JSON cache of parsed ApiSpec — not raw spec files; rkyv deferred as premature optimization"
  - "Atomic write with PID suffix for concurrent write safety"
  - "Cache key validation rejects path traversal (/, \\, ..)"
  - "SpecCache::new() returns Result — fails early on directory creation issues"
  - "Bundled specs are minimal placeholders (~200 bytes each) — full specs fetched on first real use"
  - "Version change detection via has_version_changed() with tracing::info logging"

patterns-established:
  - "SpecLoader as the canonical way to get an ApiSpec — downstream code uses this, not parsers directly"
  - "Product enum as the canonical product identifier — maps to CLI prefix, spec URL, cache key"
  - "Serve-stale via load_or_stale() — prefer expired cache over bundled for better UX"

duration: ~10min
started: 2026-03-21T08:15:00Z
completed: 2026-03-21T08:25:00Z
---

# Phase 2 Plan 03: Spec Caching, Product Registry & SpecLoader Summary

**JSON file cache with TTL, 5-product registry, bundled fallback specs, and tiered SpecLoader for fast spec access.**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~10min |
| Started | 2026-03-21T08:15:00Z |
| Completed | 2026-03-21T08:25:00Z |
| Tasks | 2 completed |
| Files created/modified | 8 |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: JSON cache save and load | Pass | Save/load roundtrip with metadata (timestamp, version, etag placeholder) |
| AC-2: TTL-based cache expiration | Pass | Expired cache returns None, stale file preserved for serve-stale |
| AC-2a: Spec version change detection | Pass | cached_version() + has_version_changed() with tracing::info logging |
| AC-3: Cache invalidation | Pass | Idempotent delete, subsequent load returns None |
| AC-4: Product registry | Pass | 5 products with correct spec URLs, CLI prefixes, cache keys |
| AC-5: SpecLoader tiered loading | Pass | Cache → bundled fallback, saves bundled to cache after load |
| AC-6: Bundled fallback specs | Pass | All 5 bundled specs valid and parseable by parse_spec() |

## Accomplishments

- SpecCache with JSON file storage, TTL, atomic writes (PID suffix), path traversal protection, version change detection (15 tests)
- Product registry mapping 5 Atlassian products to spec metadata (9 tests)
- SpecLoader with tiered loading: cache → bundled → error, serve-stale support
- 5 bundled minimal fallback specs compiled into binary via include_str!
- Total: 94 tests passing (70 existing + 24 new)

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `src/spec/cache.rs` | Created | SpecCache: JSON file cache with TTL, version detection (15 tests) |
| `src/spec/registry.rs` | Created | Product enum, ProductInfo, SpecLoader, bundled_spec() (9 tests) |
| `src/spec/bundled/jira-platform.json` | Created | Minimal bundled OpenAPI 3.0.1 spec for Jira |
| `src/spec/bundled/jira-software.json` | Created | Minimal bundled OpenAPI 3.0.1 spec for Jira Software |
| `src/spec/bundled/confluence.json` | Created | Minimal bundled OpenAPI 3.0.1 spec for Confluence |
| `src/spec/bundled/jira-service-management.json` | Created | Minimal bundled OpenAPI 3.0.1 spec for JSM |
| `src/spec/bundled/bitbucket.json` | Created | Minimal bundled Swagger 2.0 spec for Bitbucket |
| `src/spec/mod.rs` | Modified | Added cache + registry modules, re-exports for SpecCache, Product, SpecLoader |
| `Cargo.toml` | Modified | Added chrono 0.4 with serde feature |

## Decisions Made

| Decision | Rationale | Impact |
|----------|-----------|--------|
| JSON cache of parsed ApiSpec, not raw spec | ApiSpec is small (~1-5KB vs 2.47MB raw). JSON deser <5ms, well within 30ms target. rkyv adds complexity for minimal gain. | Can add rkyv later if profiling shows need |
| Minimal bundled specs (~200 bytes each) | Keep binary small. Full specs fetched on first real use via network (Phase 5). | Users get CLI structure but no real operations until first fetch |
| chrono for timestamps | Well-established Rust datetime crate, serde integration, needed for TTL math | Standard choice, no controversy |

## Deviations from Plan

None — plan executed exactly as written (including audit-added requirements).

## Issues Encountered

| Issue | Resolution |
|-------|------------|
| `/dev/null/impossible/path` test failed on Windows | Changed to null-byte path `\0invalid\0...` which fails on all OSes |

## Next Phase Readiness

**Ready:**
- SpecLoader ready for Phase 3 (Dynamic Command Tree) — load spec, build commands
- Product::from_cli_prefix ready for Phase 3 product routing (argv[1] → Product)
- Cache infrastructure ready for Phase 5 network fetch integration (cache.save after HTTP download)
- Version change detection ready for Phase 5 background refresh (check before saving new spec)
- etag field in CacheMetadata ready for Phase 5 ETag-based refresh

**Concerns:**
- None

**Blockers:**
- None

---
*Phase: 02-openapi-spec-engine, Plan: 03*
*Completed: 2026-03-21*
