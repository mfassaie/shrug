# Enterprise Plan Audit Report

**Plan:** .paul/phases/04-authentication-profiles/04-01-PLAN.md
**Audited:** 2026-03-21
**Verdict:** Enterprise-ready after fixes applied

---

## 1. Executive Verdict

Conditionally acceptable — now enterprise-ready after applying 2 must-have and 2 strongly-recommended fixes. The plan had a dual-source-of-truth design flaw for default profile tracking and missing error specifications for not-found cases. Both are corrected. I would approve this plan for production.

## 2. What Is Solid

- **Clean scope separation:** Profiles store only metadata (name, site, email, auth_type). No secrets, no credentials. This is the correct layering — credential storage is explicitly deferred to 04-02.
- **One-file-per-profile storage pattern:** Human-readable, avoids merge conflicts, each profile independently loadable. Good for CLI tools where config is user-inspectable.
- **Input validation specified:** Name format, site normalization, email validation, duplicate detection. Covers the input boundary.
- **Profile resolution chain:** `--profile flag > SHRUG_PROFILE env > config default_profile` matches the established layered precedence from Phase 1's config system. Consistent design.
- **Explicit boundaries:** Clear scope limits prevent scope creep into credential storage, OAuth, or HTTP injection.

## 3. Enterprise Gaps Identified

1. **Dual source of truth for default profile.** Original plan stored `is_default: bool` in every profile TOML file while `ShrugConfig` also has `default_profile: Option<String>`. This creates divergence: two file writes needed to change default (clear old, set new), crash between writes leaves zero defaults, and two systems can disagree on which profile is default.

2. **Missing error specifications for not-found cases.** `get`, `delete`, `show`, `use` on a non-existent profile had no specified behavior. The plan only covered happy paths. In enterprise use, clear error codes (exit code 4 = NOT_FOUND) are required for scripting and automation.

3. **No atomic file write pattern.** Profile TOML writes using direct `fs::write` can produce partial/corrupted files on crash or disk-full. A corrupted profile file makes all profile operations fail.

4. **`list()` fails entirely on single corrupted file.** If one profile TOML is corrupted (manual edit, partial write), the entire list operation fails. Users can't manage any profiles until they manually find and fix/delete the corrupted file.

## 4. Upgrades Applied to Plan

### Must-Have (Release-Blocking)

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | Dual source of truth for default profile | Objective, Task 1 (Profile struct, ProfileStore methods) | Removed `is_default` from Profile struct. Added `.default` file pattern: single plain-text file in profiles dir stores default profile name. `set_default` is now a single atomic file write. Added `clear_default()` and `default_path()` methods. |
| 2 | Missing not-found error cases | AC-3b, AC-4b (new), Task 1 tests | Added AC-3b and AC-4b for not-found error behavior. Specified `ShrugError::NotFound` (exit code 4) for get/delete/show/use on non-existent profiles. Added 4 new tests for not-found cases. |

### Strongly Recommended

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | No atomic file write pattern | Task 1 action item 6 | Added requirement for write-to-temp-then-rename pattern for all profile TOML writes. |
| 2 | `list()` fails on single corrupted file | Task 1 action item 7, tests | Added requirement for graceful skip with `tracing::warn!` on corrupted files. Added test for corrupted file skip. |

### Deferred (Can Safely Defer)

| # | Finding | Rationale for Deferral |
|---|---------|----------------------|
| 1 | File permissions (0600) on profile files | Profiles are in user's config dir, contain only email/site (not secrets). Credential files in 04-02 will need restricted permissions. |
| 2 | Profile name stored in both filename and TOML content | Low risk of divergence since ProfileStore controls writes. Can add filename-vs-content validation in a future hardening pass. |

## 5. Audit & Compliance Readiness

- **Audit evidence:** Profile CRUD operations produce clear file artifacts (one TOML per profile, one `.default` file). All state is inspectable on disk.
- **Silent failure prevention:** Corrupted files now logged with warnings rather than silently breaking. Not-found cases produce typed errors with correct exit codes for scripting.
- **Post-incident reconstruction:** Single-file-per-profile makes it trivial to see what was configured at any point. No opaque binary state.
- **Ownership:** Clear — profiles live in user's config directory, managed exclusively by `ProfileStore`.

## 6. Final Release Bar

**What must be true:**
- Default profile stored in single `.default` file, not per-profile flags
- Not-found errors use `ShrugError::NotFound` with exit code 4
- File writes use atomic temp-then-rename pattern
- `list()` is resilient to individual corrupted files

**Remaining risks if shipped as-is:** None after fixes applied. The plan is appropriately scoped for profile metadata management.

**Sign-off:** I would sign my name to this system after the applied fixes are implemented correctly.

---

**Summary:** Applied 2 must-have + 2 strongly-recommended upgrades. Deferred 2 items.
**Plan status:** Updated and ready for APPLY

---
*Audit performed by PAUL Enterprise Audit Workflow*
*Audit template version: 1.0*
