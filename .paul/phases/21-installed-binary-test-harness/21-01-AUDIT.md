# Enterprise Plan Audit Report

**Plan:** .paul/phases/21-installed-binary-test-harness/21-01-PLAN.md
**Audited:** 2026-03-23
**Verdict:** Conditionally acceptable (now upgraded to enterprise-ready after applying findings)

---

## 1. Executive Verdict

Conditionally acceptable prior to remediation. The plan had a well-defined scope, clear acceptance criteria, and appropriate boundaries protecting existing code. However, it contained a logic inconsistency in binary discovery (panic vs skip), missing environment isolation for offline tests, ambiguous dependency guidance, and incomplete files_modified metadata.

After applying 3 must-have and 3 strongly-recommended fixes, the plan is enterprise-ready. I would approve this for production.

## 2. What Is Solid

- **Separation of offline and online test modes.** The dual-mode design (skip_unless_binary for offline, skip_unless_e2e for online) is correctly layered. Offline tests can run in CI without credentials, catching regressions without API access. This is the right architecture.
- **Reuse of assert_cmd for binary execution.** Using assert_cmd::Command::new(path) instead of raw std::process::Command gives timeout handling, output capture, and consistent assertion patterns. Avoids reinventing what the crate already provides.
- **Boundaries section.** Explicitly protecting tests/e2e/*, src/*, and tests/integration.rs prevents scope creep. The "no snapshot testing" scope limit correctly defers Phase 23 work.
- **ResourceTracker with Drop.** Panic-safe cleanup using the existing LIFO pattern is the right approach for tests that create live resources.
- **Minimal validation tests.** Two tests (--version, --help) to prove the harness works without overreaching into Phase 22-24 scope.

## 3. Enterprise Gaps Identified

1. **SmokeConfig::resolve() panics but skip_unless_binary!() needs graceful failure.** The macro must return early (skip the test), not crash the test runner. A try-pattern is required.

2. **No environment variable isolation for offline tests.** If a developer has SHRUG_SITE/SHRUG_EMAIL/SHRUG_API_TOKEN set in their shell, offline tests would silently pass credential env vars to the binary. An "offline" test that accidentally hits a live API is a correctness and security risk.

3. **files_modified frontmatter missing Cargo.toml.** Task 2 explicitly modifies Cargo.toml (dev-deps, [[test]] section) but it wasn't listed. This breaks conflict detection if parallel plans touch Cargo.toml.

4. **Ambiguous timeout approach.** The plan mentioned wait_timeout crate, then contradicted itself by recommending assert_cmd. Ambiguous instructions risk the implementer picking the wrong approach.

5. **Conditional [[test]] section guidance.** The plan said "if needed" and "check if cargo auto-discovers" for a case that is definitively required. Directory-based test targets always need explicit [[test]] entries.

6. **Validation tests in main.rs vs module.** Plan was indecisive ("directly in main.rs or a dedicated validation.rs module"). The existing e2e suite uses separate modules per concern. Consistency requires a dedicated module.

## 4. Upgrades Applied to Plan

### Must-Have (Release-Blocking)

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | resolve() panic incompatible with skip macro | Task 1 action | Added try_resolve() returning Option<SmokeConfig>. resolve() now wraps try_resolve(). Skip macros use try_resolve(). |
| 2 | No env var isolation for offline mode | Task 1 action, AC-7 added | run() in offline mode calls .env_remove() for SHRUG_SITE, SHRUG_EMAIL, SHRUG_API_TOKEN. New AC-7 validates this. |
| 3 | files_modified missing Cargo.toml | Frontmatter | Added Cargo.toml and tests/smoke/validation.rs to files_modified list. |

### Strongly Recommended

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | Ambiguous timeout approach (wait_timeout vs assert_cmd) | Task 1 action | Removed wait_timeout reference. Definitive: use assert_cmd::Command::new() only. |
| 2 | Conditional [[test]] guidance | Task 2 action | Made definitive: [[test]] IS required for directory-based targets. Added explanation. |
| 3 | Indecisive validation test location | Task 2 action, frontmatter | Changed to dedicated tests/smoke/validation.rs module. Added to files_modified. |

### Deferred (Can Safely Defer)

| # | Finding | Rationale for Deferral |
|---|---------|----------------------|
| 1 | Duplicate E2eConfig struct across test targets | Rust test binaries can't share code without a shared crate. Duplication is pragmatic for test harnesses. Can extract a shared test-utils crate later if the duplication grows. |

## 5. Audit & Compliance Readiness

- **Audit evidence:** The plan produces verifiable artifacts (test binaries, test output). All acceptance criteria are testable via cargo commands.
- **Silent failure prevention:** The try_resolve/skip pattern ensures tests skip visibly rather than silently succeeding or crashing. env_remove prevents offline tests from accidentally hitting live APIs.
- **Post-incident reconstruction:** If a smoke test fails in CI, the RunResult captures stdout, stderr, and exit code. Diagnostic messages in skip macros explain why tests were skipped.
- **Ownership:** Clear file ownership (4 new files, 1 modified). No shared mutable state between test targets.

## 6. Final Release Bar

**What must be true before this plan ships:**
- skip_unless_binary!() gracefully skips (not panics) when binary absent
- Offline SmokeRunner removes credential env vars from child process
- Both validation tests pass against shrug.exe on PATH
- All 529 existing tests continue to pass
- cargo clippy clean, cargo fmt clean

**Risks if shipped as-is (pre-audit):**
- Tests could crash entire test suite if binary not on PATH (panic in resolve)
- Offline tests could silently hit live API if credentials in environment
- Cargo.toml conflict risk if parallel plans modify it

**Sign-off:** After applying the 6 findings above, I would sign my name to this plan.

---

**Summary:** Applied 3 must-have + 3 strongly-recommended upgrades. Deferred 1 item.
**Plan status:** Updated and ready for APPLY

---
*Audit performed by PAUL Enterprise Audit Workflow*
*Audit template version: 1.0*
