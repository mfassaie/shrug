# Enterprise Plan Audit Report

**Plan:** .paul/phases/01-project-foundation/01-02-PLAN.md
**Audited:** 2026-03-21
**Verdict:** Enterprise-ready after fixes applied

---

## 1. Executive Verdict

**Conditionally acceptable → Enterprise-ready after applying 3 must-have + 3 strongly-recommended fixes.**

The plan demonstrates solid architectural thinking: layered config precedence is correctly ordered, platform path abstraction via `directories` is the right call, and the scope boundaries are disciplined (no auth, no behavior changes). However, several gaps would cause real problems in production: undefined merge semantics would silently reset config values, untested env var parsing would panic on bad input, and parallel test pollution would create flaky CI.

After applying the 6 fixes below, I would approve this plan for production.

## 2. What Is Solid

- **Layered precedence order** (defaults < user < project < env < CLI) matches industry standard (12-factor app compatible). Correctly prioritized.
- **Scope discipline** is excellent: no auth, no behavior changes, no new deps. This prevents the config system from becoming a kitchen sink.
- **Platform paths via `directories` crate** is the correct abstraction — avoids hardcoded paths and handles XDG, Windows AppData, and macOS Library correctly.
- **Separation of CLI parsing from config loading** — clap handles CLI, config handles files/env, merge happens after both. Clean separation.
- **Boundaries protect completed work** (exit_codes.rs, ci.yml) from modification. Correct.
- **TOML-only format** avoids the complexity of multi-format config parsing. Right decision for v0.1.

## 3. Enterprise Gaps Identified

1. **Merge semantics undefined:** "overwrite non-None fields" is ambiguous with serde. If a TOML file is deserialized directly into `ShrugConfig`, absent fields get `Default::default()` values, which would silently reset values from a prior layer. This is a data-loss bug in the config system.

2. **Env var parsing can panic:** `SHRUG_PAGE_SIZE="abc"` will cause `.parse::<u32>().unwrap()` to panic. No acceptance criterion covers this path.

3. **Project config walk stopping condition undefined:** "walk up to git root or filesystem root" — what is "git root"? Does it check for `.git` file (submodules) or only `.git` directory? What if neither exists? The plan needs a deterministic stopping condition.

4. **Env var test pollution:** Tests that set `std::env::set_var("SHRUG_OUTPUT", "json")` will leak into other tests running in parallel (Rust tests are multi-threaded by default). This creates flaky CI.

5. **ConfigError lacks file path context:** AC-6 says "identifying the file and parse error" but the existing `ShrugError::ConfigError(String)` is a plain string. Nothing enforces that the file path is included. The plan should specify that the error message format includes the failing path.

6. **No Serialize derive on ShrugConfig:** Without Serialize, you can't log the resolved config at debug level, can't implement `shrug config show`, and can't include config state in error reports. Adding Serialize now costs nothing; retrofitting later touches every field.

## 4. Upgrades Applied to Plan

### Must-Have (Release-Blocking)

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | Merge semantics undefined — absent TOML keys reset prior layer values | Task 1 action (item 4) | Added: use `ShrugConfigPartial` with all `Option<T>` fields for deserialization, merge non-None fields per layer |
| 2 | Env var parsing can panic on invalid values | Task 1 action (item 3e), AC-7 added | Added: validate env var types, return ConfigError on invalid value; new AC-7 for invalid env var |
| 3 | Project config walk stopping condition undefined | Task 1 action (item 2), AC-8 added | Added: explicit stopping condition (stop at .git dir or filesystem root); new AC-8 |

### Strongly Recommended

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | Env var tests will pollute each other in parallel | Task 2 action (test list) | Added: note requiring serial test execution or env var isolation |
| 2 | ConfigError should include file path in message | Task 2 action, verification | Strengthened: "with file path in message" in test requirements |
| 3 | ShrugConfig should derive Serialize | Task 1 action (item 6) | Added: derive Serialize + Deserialize on ShrugConfig and cli enums |

### Deferred (Can Safely Defer)

| # | Finding | Rationale for Deferral |
|---|---------|----------------------|
| 1 | Config file creation/init command (`shrug config init`) | Not needed until user-facing config editing is required; manual file creation is acceptable for v0.1 |
| 2 | Config value validation (e.g. page_size > 0, cache_ttl > 0) | Sensible defaults prevent zero values in practice; validation can be added when values drive behavior in later phases |

## 5. Audit & Compliance Readiness

- **Audit evidence:** Config loading is deterministic and testable. The layered merge with explicit partial struct creates a clear paper trail of which layer set which value (traceable if debug logging is added).
- **Silent failure prevention:** AC-6 (invalid TOML) and AC-7 (invalid env var) ensure config errors surface clearly rather than falling back to defaults silently.
- **Post-incident reconstruction:** With Serialize on ShrugConfig, debug-level logging can dump the fully-resolved config at startup, enabling post-incident analysis of "what config was active when this happened."
- **Ownership:** Config module is self-contained in src/config.rs with clear public API (load_config, ShrugConfig, ShrugPaths).

## 6. Final Release Bar

**What must be true before this plan ships:**
- Merge semantics use partial struct overlay (not direct deserialize overwrite)
- Invalid env vars produce errors, not panics
- Project config walk has deterministic stopping at .git or filesystem root
- Env var tests are isolated from each other

**Remaining risks if shipped as-is:** Minimal after fixes applied. The config system is intentionally simple and will be extended by later phases.

**Sign-off:** With the 6 applied fixes, I would sign my name to this system.

---

**Summary:** Applied 3 must-have + 3 strongly-recommended upgrades. Deferred 2 items.
**Plan status:** Updated and ready for APPLY

---
*Audit performed by PAUL Enterprise Audit Workflow*
*Audit template version: 1.0*
