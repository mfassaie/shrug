# Enterprise Plan Audit Report

**Plan:** `.paul/phases/22-static-commands-global-flags/22-01-PLAN.md`
**Audited:** 2026-03-23
**Verdict:** Conditionally acceptable

---

## Executive Verdict

The plan is well-structured and achieves its stated goal of offline smoke test coverage for static commands and global flags. It builds correctly on the Phase 21 harness and respects the boundary constraints. However, it has three must-fix gaps around profile cleanup reliability and parallel test safety that would cause flaky failures in CI, plus several specificity gaps that would leave the implementer guessing. After the upgrades below are applied, the plan is fit for execution.

---

## What Is Solid

1. **Correct use of the Phase 21 harness.** The plan uses `skip_unless_binary!`, `SmokeRunner::new()` (offline mode), and `env_remove` credential isolation exactly as designed. No reinvention.

2. **Scope discipline.** Boundaries are explicit. No source code changes, no live API calls, no golden-file testing (correctly deferred to Phase 23). The `tests/smoke/harness.rs` and `fixtures.rs` are marked read-only.

3. **Test naming convention.** The `unique_name()` pattern with process-ID suffix matches the existing e2e/auth.rs pattern, avoiding profile name collisions between parallel test threads.

4. **Coverage breadth.** All five output formats, all three colour modes, three verbosity levels, completions for all four shells, and the full profile CRUD lifecycle are addressed. The ~25 test count is reasonable.

5. **Verification checklist.** The final verification section covers clippy, full suite regression, and individual feature areas. Good.

---

## Enterprise Gaps

### MUST-HAVE (Release-Blocking)

**M1. No panic-safe cleanup for profile tests.**
The plan says "All tests clean up profiles on completion" but uses a `create_test_profile` / `delete_test_profile` helper pair. If an assertion panics between create and delete, the profile leaks onto the filesystem. The existing `ResourceTracker` in `fixtures.rs` implements `Drop` for exactly this scenario, but the plan says `fixtures.rs` is read-only and cannot be used (it requires `E2eConfig`). The plan must specify a `Drop`-based guard for profile cleanup. A simple `ProfileGuard` struct that holds the runner reference and profile name, calling `delete_test_profile` in its `Drop` impl, would suffice.

**M2. `unique_name()` uses only `process::id()`, not thread ID.**
The plan specifies `--test-threads=1` in verify commands, but Task 2's verify step 2 runs `cargo test --test smoke -- --test-threads=1` while step 3 runs bare `cargo test` (full suite, default parallelism). If the full suite runs smoke tests in parallel with other tests that create profiles (e2e/auth.rs also uses `unique_name` with the same PID), names could collide. The unique_name function must include a module-specific prefix (e.g., `smoke-t1-{pid}` vs `smoke-t2-{pid}`) and/or a thread ID component, or the plan must mandate `--test-threads=1` for the full-suite verify step too.

**M3. `test_profile_use_sets_default` leaves .default file pointing to a test profile.**
When `profile use --name <test-profile>` is called, it writes the test profile name into the `.default` file. If the test deletes the profile but doesn't restore the previous default (or clear the `.default` file), subsequent runs of shrug on the same machine may fail with "default profile not found". The cleanup sequence must capture the pre-existing default, and restore it after deleting test profiles.

### STRONGLY RECOMMENDED

**S1. Missing env var isolation for `SHRUG_OUTPUT`, `SHRUG_COLOR`, `SHRUG_PROFILE`.**
`SmokeRunner::new()` removes `SHRUG_SITE`, `SHRUG_EMAIL`, `SHRUG_API_TOKEN` (confirmed in harness.rs lines 201-206). But it does NOT remove `SHRUG_OUTPUT`, `SHRUG_COLOR`, or `SHRUG_PROFILE`. If any of these are set in the developer's shell, they will override the flags being tested. For example, `SHRUG_OUTPUT=yaml` would make `test_output_json` pass the flag but get YAML output. The plan should specify removing these env vars too, or note that the runner needs extending. Since the boundary says harness.rs is read-only, the tests themselves must call `.env_remove()` on these variables.

**S2. `test_auth_status_no_profile` expected behaviour is underspecified.**
AC-2 says "auth status without a profile returns a non-zero exit code with guidance". But on a developer machine with existing profiles, there IS a default profile. The test may get exit 0 with valid status, not the expected non-zero. The plan needs to either (a) create a temporary config dir via `SHRUG_CONFIG_DIR` or `XDG_CONFIG_HOME` env var override to isolate from real profiles, or (b) accept that this test may pass or skip on machines with existing profiles, with explicit skip logic.

**S3. Argument ordering for global flags is inconsistent.**
Task 2 shows `--output json profile list` (global flag before subcommand) while some test cases show `--color auto --help`. The plan should confirm that shrug accepts global flags both before and after subcommands, or standardise the argument position. Based on the cli.rs `global = true` annotations, clap should accept both positions, but the plan should be explicit about which convention tests use and why.

**S4. `test_dry_run_with_help` does not actually test `--dry-run` behaviour.**
The plan tests `--dry-run --help`, which will show help text regardless of `--dry-run`. This tests that clap accepts the flag without error, but not that `--dry-run` changes behaviour. A better test would be `--dry-run profile list`, asserting exit 0 and possibly checking that "DRY RUN" appears in output (matching the existing e2e pattern in features.rs lines 131-133). The `--help` variant can be kept as a separate parse-acceptance test.

**S5. No test for invalid/unknown flag rejection.**
A negative test (e.g., `--output xml profile list` should fail with non-zero exit) would guard against flag validation regressions. At least one negative case should be included.

### CAN SAFELY DEFER

**D1. No test for `--page-all`, `--limit`, `--markdown`, `--json`, or JQL shorthand flags.**
These require live API responses to exercise meaningfully. Correctly deferred to Phase 24.

**D2. No test for `_complete` hidden subcommand.**
Dynamic completions are Phase 20 territory. Correct to exclude here.

**D3. No test for `cache refresh` (requires network).**
Correctly scoped to `cache --help` only.

**D4. No assertion on completions output content.**
The plan checks non-empty stdout for each shell. Checking for a specific string (e.g., "profile" or "jira" appearing in the completion script) would catch regressions where completions silently produce garbage. This is a nice-to-have and can go to Phase 23 golden-file testing.

---

## Upgrades Applied

| ID | Category | Change | Location |
|----|----------|--------|----------|
| M1 | Must-have | Added `ProfileGuard` Drop-based cleanup specification | Task 1 helper pattern |
| M2 | Must-have | Changed `unique_name` to include module-scoped prefix | Task 1 and Task 2 helper patterns |
| M3 | Must-have | Added default profile save/restore in `test_profile_use_sets_default` | Task 1 test description |
| S1 | Strongly recommended | Added `env_remove` calls for `SHRUG_OUTPUT`, `SHRUG_COLOR`, `SHRUG_PROFILE` | Task 1 and Task 2 action sections |
| S2 | Strongly recommended | Replaced `test_auth_status_no_profile` with `test_auth_status_with_profile` | Task 1 test description |
| S3 | Strongly recommended | Added explicit note on global flag positioning convention | Task 2 action section |
| S4 | Strongly recommended | Changed `test_dry_run_with_help` to `test_dry_run_profile_list` | Task 2 test description |
| S5 | Strongly recommended | Added `test_invalid_output_format_rejected` negative test | Task 2 test description |

---

## Audit and Compliance

| Check | Status |
|-------|--------|
| All acceptance criteria testable | Pass |
| No credential leakage paths | Pass (env_remove in offline runner) |
| Cleanup on panic | Fixed (M1) |
| Parallel safety | Fixed (M2) |
| Default profile corruption | Fixed (M3) |
| Env var isolation | Fixed (S1) |
| Boundary compliance | Pass (no src changes, no harness changes) |
| Scope discipline | Pass (no network, no golden files) |
| Verify steps executable | Pass |

---

## Final Release Bar

The plan meets the release bar after the eight upgrades above are applied. The three must-have items (M1, M2, M3) are non-negotiable. They prevent flaky CI failures and developer-machine state corruption. The five strongly-recommended items (S1-S5) prevent false positives in test results and improve diagnostic value.

**Summary counts:** 3 must-have, 5 strongly recommended, 4 can safely defer. 8 upgrades applied to plan.
