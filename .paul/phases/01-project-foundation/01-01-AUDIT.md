# Enterprise Plan Audit Report

**Plan:** .paul/phases/01-project-foundation/01-01-PLAN.md
**Audited:** 2026-03-21
**Verdict:** Conditionally Acceptable → **Enterprise-ready after applied fixes**

---

## 1. Executive Verdict

**Conditionally acceptable**, upgraded to enterprise-ready after applying 4 must-have and 3 strongly-recommended fixes.

The original plan had a clean structure and sensible task decomposition, but contained a build-breaking configuration error (edition 2024), lacked unit tests for critical error infrastructure, had an unspecified error-to-exit-code bridge in main.rs, and an unpinned CI toolchain. All issues have been remediated in the updated plan.

Would I approve this plan for production if I were accountable? **Yes, after the applied fixes.**

## 2. What Is Solid

- **Task decomposition:** Three tasks are well-separated by concern (scaffold, CLI skeleton, infrastructure). No coupling or dependency confusion.
- **Boundary definition:** Explicit scope limits prevent premature work. "No config loading" and "No API calls" are correct guardrails for Phase 1.
- **Verification checklist:** Covers build, test, lint, format, and runtime behavior — comprehensive for a foundation plan.
- **Dependency discipline:** Only Phase 1 dependencies included. Deferred crates (keyring, rkyv, etc.) explicitly called out as out-of-scope.
- **Exit code taxonomy:** Covers the full error space for an API client CLI (auth, not-found, rate-limited, network, server). Appropriate granularity.

## 3. Enterprise Gaps Identified

1. **Build-breaking: Rust edition "2024"** — Edition 2024 requires Rust 1.85+ which is very recent. CI runners on ubuntu-latest/macos-latest may not have it. A build failure in CI on day one destroys confidence.

2. **No unit tests** — "cargo test passes (even if no tests yet)" is zero validation. The error → exit code mapping is critical infrastructure that must be tested from the start. Untested foundation code propagates silently into all future phases.

3. **Error type naming: "ShurgError"** — Typo. The project is "shrug", not "shurg". This would embed a wrong name throughout the codebase.

4. **No Rust toolchain pinning** — Without `rust-toolchain.toml`, CI runners and developers may use different Rust versions. This causes "works on my machine" failures.

5. **Unspecified error bridge in main.rs** — "Use process::exit with appropriate exit code on error" doesn't define how anyhow::Result or ShrugError actually reaches the exit code. The bridge function pattern must be explicit.

6. **CI caching strategy vague** — "Cache cargo registry and target directory" doesn't specify which action or key strategy. Wrong caching breaks CI reproducibility.

7. **No guidance on error Display messages** — thiserror's `#[error("...")]` messages are what users see. Without guidance, they'll be developer-facing jargon instead of actionable messages.

## 4. Upgrades Applied to Plan

### Must-Have (Release-Blocking)

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | Edition 2024 may break CI | Task 1 action | Changed to edition = "2021"; added rust-toolchain.toml with stable channel; removed rust-version = "1.85" |
| 2 | Error type typo "ShurgError" | All occurrences | Corrected to "ShrugError" throughout plan |
| 3 | No unit tests for error infrastructure | Task 3 action, AC-3, verification | Added requirement for unit tests covering every ShrugError variant → exit code mapping and Display output |
| 4 | No Rust toolchain pinning | Task 1 action, files_modified, AC-4 | Added rust-toolchain.toml to deliverables; CI now reads toolchain from this file |

### Strongly Recommended

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | Unspecified error-to-exit-code bridge | Task 2 action, AC-5 (new) | Added explicit `run()` → `Result<(), ShrugError>` pattern with single exit point; added AC-5 |
| 2 | CI caching strategy vague | Task 3 action | Specified `dtolnay/rust-toolchain@stable` + `Swatinem/rust-cache@v2`; split into fast-check + build-test jobs |
| 3 | No Display message guidance | Task 3 action | Added requirement for user-facing, actionable `#[error("...")]` messages with examples |

### Deferred (Can Safely Defer)

| # | Finding | Rationale for Deferral |
|---|---------|----------------------|
| 1 | Edition 2024 upgrade | Safe to defer to a later phase once CI confirms Rust 1.85+ availability on all runners. Edition 2021 is fully capable for MVP. |
| 2 | Integration tests for CLI binary (assert_cmd) | Dev-dependencies are included but no integration tests required in this plan. Phase 8 has dedicated integration test plan. Foundation unit tests are sufficient for Phase 1. |

## 5. Audit & Compliance Readiness

**Audit evidence:** The plan now produces testable artifacts — unit tests prove error-to-exit-code mapping is correct. CI produces build/test results on all three platforms. Toolchain is pinned for reproducibility.

**Silent failure prevention:** The single-exit-point pattern in main.rs ensures all errors flow through the same display + exit path. No error can silently swallow an exit code.

**Post-incident reconstruction:** Tracing subscriber with env-filter means verbose logs can be enabled retroactively (`RUST_LOG=debug`). Error Display messages provide actionable context.

**Ownership:** CI pipeline ensures every push is validated. Clippy warnings are treated as errors (`-D warnings`), preventing code quality drift.

## 6. Final Release Bar

**What must be true before this plan ships:**
- All ShrugError variants have unit-tested exit code mappings
- rust-toolchain.toml pins stable Rust
- CI has two jobs: fast-check (lint) and build-test (3-platform matrix)
- main.rs has a single `run()` → exit code bridge

**Risks remaining if shipped as-is:** None after applied fixes. This is a foundation plan with no external-facing behavior beyond `--version` and `--help`.

**Sign-off:** I would sign my name to this system after the applied fixes.

---

**Summary:** Applied 4 must-have + 3 strongly-recommended upgrades. Deferred 2 items.
**Plan status:** Updated and ready for APPLY

---
*Audit performed by PAUL Enterprise Audit Workflow*
*Audit template version: 1.0*
