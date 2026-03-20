---
phase: 01-project-foundation
plan: 02
subsystem: infra
tags: [toml, config, directories, serde, layered-precedence]

requires:
  - phase: 01-project-foundation
    provides: Cargo scaffold, error types, CLI skeleton (plan 01-01)
provides:
  - TOML-based config system with layered precedence
  - Platform-correct paths (config, cache, data)
  - ShrugConfig struct with Serialize/Deserialize
  - Partial struct merge pattern for config layering
affects: [01-03-logging, phase-4-auth-profiles, phase-2-spec-cache]

tech-stack:
  added: []
  patterns: [partial-struct-merge for layered config, env var validation with typed errors]

key-files:
  created: [src/config.rs]
  modified: [src/cli.rs, src/main.rs, src/lib.rs]

key-decisions:
  - "ShrugConfigPartial with Option<T> fields for merge — prevents silent value reset across layers"
  - "Project config walk stops at .git boundary — prevents config leaking from parent repos"

patterns-established:
  - "Layered config: defaults < user config < project config < env vars < CLI flags"
  - "Env var validation returns ConfigError with var name — no panics on bad input"
  - "Config loaded in main.rs before command dispatch, CLI overrides applied after"

duration: ~15min
started: 2026-03-21T06:30:00Z
completed: 2026-03-21T06:45:00Z
---

# Phase 1 Plan 02: Config System Summary

**TOML-based config with layered precedence (defaults < user < project < env < CLI), platform-correct paths via directories crate, and partial struct merge pattern.**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~15min |
| Started | 2026-03-21T06:30:00Z |
| Completed | 2026-03-21T06:45:00Z |
| Tasks | 2 completed |
| Files created/modified | 4 |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: Default config with no files | Pass | ShrugConfig::default() returns table/auto/50/24 |
| AC-2: User config from platform path | Pass | directories crate resolves platform config dir |
| AC-3: Project config overrides user | Pass | Partial merge test: project CSV overrides user YAML, user page_size preserved |
| AC-4: Env vars override file configs | Pass | SHRUG_OUTPUT, SHRUG_SITE, SHRUG_PAGE_SIZE tested |
| AC-5: CLI flags override everything | Pass | apply_cli_overrides test: table overrides json |
| AC-6: Invalid TOML reports file path | Pass | Error message includes file path + parse detail |
| AC-7: Invalid env var values error cleanly | Pass | SHRUG_PAGE_SIZE="abc" returns ConfigError naming the var |
| AC-8: Project config walk stops at .git | Pass | Nested dir with .git root stops correctly |

## Accomplishments

- ShrugConfig with 6 fields, Serialize + Deserialize derives, Default impl
- ShrugConfigPartial with all Option<T> fields for safe layered merge
- ShrugPaths with platform-correct dirs (config, cache, data) and project config walk
- load_config() implementing full 4-layer precedence chain
- 13 new unit tests covering all ACs including error paths

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `src/config.rs` | Created | Config struct, partial merge, paths, env overrides, load_config (447 lines) |
| `src/cli.rs` | Modified | Added Serialize/Deserialize/Debug/PartialEq derives + serde rename on OutputFormat, ColorChoice |
| `src/main.rs` | Modified | Config loading before dispatch, CLI overrides, config passed to run() |
| `src/lib.rs` | Modified | Added `pub mod config` |

## Decisions Made

| Decision | Rationale | Impact |
|----------|-----------|--------|
| ShrugConfigPartial for merge | Direct deserialize into full struct resets absent fields to defaults, breaking layered precedence | Pattern reusable for future config sections |
| Env var validation with typed errors | Prevents panics on invalid input (e.g. non-numeric page size) | All env var parse failures return actionable ConfigError |

## Deviations from Plan

### Summary

| Type | Count | Impact |
|------|-------|--------|
| Auto-fixed | 0 | None |
| Scope additions | 0 | None |
| Deferred | 0 | None |

**Total impact:** Plan executed as written. No deviations.

## Issues Encountered

None.

## Next Phase Readiness

**Ready:**
- Config system extensible — add new fields to ShrugConfig/ShrugConfigPartial + merge
- Platform paths available for cache dir (Phase 2) and data dir (Phase 4)
- CLI overrides pattern established for future flags

**Concerns:**
- None

**Blockers:**
- None

---
*Phase: 01-project-foundation, Plan: 02*
*Completed: 2026-03-21*
