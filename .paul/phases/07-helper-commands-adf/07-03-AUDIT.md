# Enterprise Plan Audit Report

**Plan:** .paul/phases/07-helper-commands-adf/07-03-PLAN.md
**Audited:** 2026-03-21
**Verdict:** Enterprise-ready after fixes applied

---

## 1. Executive Verdict

Conditionally acceptable, upgraded to enterprise-ready after applying fixes below.

A clean, well-scoped plan. Shell completions use the established clap_complete approach. Resolution caches follow the project's existing caching patterns (site-scoped, TTL-based, JSON file storage). Two gaps needed fixing: the completions function printed directly to stdout (untestable), and cache writes had no atomicity guarantee.

I would approve this plan for production after the applied fixes.

## 2. What Is Solid

- **clap_complete already a dependency**: No new dependencies needed. Standard approach for Rust CLIs.
- **Site-scoped caches via SHA-256 hash**: Safe, no path traversal risk from URL characters. 16-char prefix gives sufficient collision resistance.
- **Separate populate/resolve methods**: Clean separation. Caches can be populated by future commands without coupling resolution logic to API calls.
- **Case-insensitive matching**: Correct for both field names ("Story Points" vs "story points") and user names.
- **24h TTL matching spec cache**: Consistent behaviour across all cache types.
- **Boundaries protect existing code**: Helpers not modified, resolution not wired in yet (deferred correctly).

## 3. Enterprise Gaps Identified

**Gap 1 — generate_completions prints to stdout directly (severity: must-have)**
The plan specified `generate_completions(shell: &str) -> Result<(), ShrugError>` which prints to stdout. Unit tests cannot verify the output without capturing stdout, which is fragile. `clap_complete::generate()` accepts a `&mut impl Write` parameter. The function should accept a writer so tests can pass a `Vec<u8>` buffer.

**Gap 2 — Non-atomic cache writes (severity: strongly-recommended)**
The plan's `save_cache` writes JSON directly to the cache file. If the process is killed mid-write (Ctrl+C, OOM, disk full), the cache file would be corrupted. The next read would fail silently (returning None) which is graceful, but could also parse partially-written JSON as valid data with wrong values. Write-to-temp-then-rename is the standard atomic write pattern and is already used elsewhere in the codebase (profile writes in auth/profile.rs).

## 4. Upgrades Applied to Plan

### Must-Have (Release-Blocking)

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | generate_completions needs writer parameter for testability | Task 1 action (function signature), Task 1 action (main.rs wiring) | Changed signature to accept `&mut impl Write`, main.rs passes `&mut std::io::stdout()` |

### Strongly Recommended

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | Atomic cache writes to prevent corruption | Task 2 action (save_cache) | Added write-to-temp-then-rename instruction |

### Deferred (Can Safely Defer)

| # | Finding | Rationale for Deferral |
|---|---------|----------------------|
| 1 | TTL source (updated_at JSON field vs file mtime) not specified | Either approach works. Non-fresh caches return None, so the worst case is an unnecessary re-fetch. |

## 5. Audit & Compliance Readiness

- **Audit evidence**: Completions produce deterministic output testable via writer parameter. Cache operations have round-trip tests.
- **Silent failure prevention**: Missing/expired caches return None. Atomic writes prevent corrupted-cache edge cases.
- **Post-incident reconstruction**: Cache files are human-readable JSON. SHA-256 site hash is deterministic.
- **Ownership**: Two self-contained modules, no cross-module coupling.

## 6. Final Release Bar

**What must be true before shipping:**
- generate_completions accepts a writer parameter
- save_cache uses atomic writes (temp file + rename)
- All four shells produce non-empty valid output

**Remaining risks if shipped as-is (after fixes):**
- Resolution caches are not auto-populated (by design, scope limit). Users must populate them manually or wait for a future command.
- No validation that cache entries contain sensible values (e.g., customfield IDs matching expected patterns). Low risk since populate() is called by trusted code.

I would sign my name to this plan with the applied fixes.

---

**Summary:** Applied 1 must-have + 1 strongly-recommended upgrade. Deferred 1 item.
**Plan status:** Updated and ready for APPLY

---
*Audit performed by PAUL Enterprise Audit Workflow*
*Audit template version: 1.0*
