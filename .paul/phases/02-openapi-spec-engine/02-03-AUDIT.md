# Enterprise Plan Audit Report

**Plan:** .paul/phases/02-openapi-spec-engine/02-03-PLAN.md
**Audited:** 2026-03-21
**Verdict:** Enterprise-ready after fixes applied

---

## 1. Executive Verdict

**Conditionally acceptable → enterprise-ready after applying 3 must-have + 2 strongly-recommended fixes.**

The plan demonstrates sound caching architecture: tiered loading, serve-stale pattern, atomic writes, and clean separation between cache mechanics (SpecCache) and domain logic (SpecLoader/Product). However, the original plan was missing spec version change detection (explicitly requested by user), had no protection against path traversal via cache keys, and had a concurrent write collision risk in the atomic write implementation.

I would approve this plan for production after the applied fixes.

## 2. What Is Solid (Do Not Change)

- **Tiered loading strategy (cache → bundled → error).** This is the correct architecture. Network fetch will slot in naturally between cache and bundled when Phase 5 adds HTTP. No redesign needed.

- **Serve-stale pattern via load_stale().** Keeping expired cache files and providing a separate method to read them is the right approach for background refresh. Users get fast responses with potentially-stale data rather than slow startup waiting for a fetch.

- **Bundled fallback specs via include_str!.** Compile-time embedding ensures first-run works with no network. Minimal placeholder specs keep binary size small. Full specs will be fetched on first real use.

- **CacheMetadata with etag placeholder.** Forward-thinking design that avoids a schema change when ETag refresh is added in Phase 5.

- **Separation of SpecCache (I/O) from SpecLoader (domain logic).** Clean testability — cache can be tested with tempdir, loader logic can be tested independently.

- **rkyv deferral decision.** JSON deserialization of the small parsed ApiSpec (~1-5KB) is well within the 30ms target. Adding rkyv now would be premature optimization with unnecessary complexity.

## 3. Enterprise Gaps Identified

### Gap 1: No spec version change detection (CRITICAL, user-requested)
CacheMetadata stores `spec_version` but the plan provides no mechanism to compare versions between cached and newly-fetched specs. When Atlassian updates their APIs, shrug needs to know the spec changed. Without version comparison:
- No way to notify users of API changes
- No audit trail of spec evolution
- Silent behavior changes when specs update

### Gap 2: Cache key path traversal (CRITICAL)
The cache key is used directly in a file path: `{cache_dir}/specs/{product}.json`. A malicious or malformed cache key like `../../etc/passwd` could write outside the cache directory. While the current Product enum uses hardcoded safe strings, the save/load methods accept `&str` directly, so any future caller could pass unsafe keys.

### Gap 3: SpecCache::new() swallows directory creation errors
The plan says `new()` creates the directory but returns `Self`, not `Result`. If the cache directory can't be created (permissions, disk full, read-only filesystem), subsequent save operations will silently fail. Constructor should return Result.

### Gap 4: Atomic write collision
The plan says "write to .tmp file first, then rename" but doesn't specify a unique tmp filename. Two concurrent shrug invocations writing the same product would collide on the .tmp file, potentially corrupting data. Using PID or random suffix prevents this.

## 4. Upgrades Applied to Plan

### Must-Have (Release-Blocking)

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | Spec version change detection — user explicitly requested | AC-2a added, Task 1 SpecCache methods added | Added `cached_version()` and `has_version_changed()` methods with tracing::info logging. Added AC-2a with Given/When/Then. Added 4 test cases. |
| 2 | Cache key path traversal protection | Task 1 save() method | Added validation: reject keys containing `/`, `\`, or `..`. Added test case. |
| 3 | SpecCache::new() error handling | Task 1 new() method | Changed return type to Result<Self, ShrugError>. Errors on directory creation failure. |

### Strongly Recommended

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | Atomic write unique tmp filename | Task 1 save() method | Specified PID or random suffix for .tmp file to prevent concurrent write collisions. |
| 2 | SpecLoader version check passthrough | Task 2 SpecLoader methods | Added `check_version()` method delegating to cache.has_version_changed(). |

### Deferred (Can Safely Defer)

| # | Finding | Rationale for Deferral |
|---|---------|----------------------|
| 1 | Cache size limits | With 5 products at ~2MB each (when full specs are cached), total is ~10MB. Not a concern until many more products or spec snapshots are retained. |
| 2 | File locking for concurrent read/write | Atomic rename handles the write case. A reader could theoretically get a partial file during rename, but this is a single-file atomic operation on all major OSes. Extremely unlikely to cause issues. |

## 5. Audit & Compliance Readiness

**Evidence production:** Cache metadata provides an audit trail: when each spec was cached, what version, and (future) what ETag. The version change detection adds tracing::info logging, producing observable evidence of spec evolution.

**Silent failure prevention:** Constructor returning Result prevents silently operating with a broken cache directory. Cache key validation prevents silent writes to unexpected paths.

**Post-incident reconstruction:** If a user reports wrong behavior, the cached spec version in metadata allows checking whether a spec change caused the issue. The version change log makes it possible to correlate behavior changes with spec updates.

**Ownership:** Two clean modules (cache.rs, registry.rs) with clear separation of concerns. Maintainable by anyone who understands the tiered loading pattern.

## 6. Final Release Bar

**What must be true before this ships:**
- All 6 original ACs + 1 audit-added AC pass
- Version change detection tested with version comparison + logging
- Cache key validation rejects path traversal attempts
- SpecCache::new() returns error on directory creation failure
- No regressions in existing 70 tests

**Remaining risks if shipped as-is (after fixes):**
- Bundled specs are minimal placeholders — users won't have full API coverage until network fetch is added (Phase 5). This is acceptable and documented.
- No cache encryption — spec files are stored as plaintext JSON. Specs are public (Atlassian publishes them), so no sensitive data concern.

**Sign-off:** With the applied fixes, I would sign my name to this plan. The version change detection addresses the user's explicit requirement, and the security/reliability fixes close the gaps that could cause silent failures.

---

**Summary:** Applied 3 must-have + 2 strongly-recommended upgrades. Deferred 2 items.
**Plan status:** Updated and ready for APPLY

---
*Audit performed by PAUL Enterprise Audit Workflow*
*Audit template version: 1.0*
