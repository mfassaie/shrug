# Plan 21-01 Summary

**Phase:** 21 — Installed Binary Test Harness
**Plan:** 01
**Status:** Complete
**Date:** 2026-03-23

## What Was Built

New `tests/smoke/` test target with a harness that runs tests against an installed `shrug.exe` binary found on PATH, supporting both offline and online test modes.

## Files Created/Modified

| File | Action |
|------|--------|
| tests/smoke/main.rs | Created — test module root |
| tests/smoke/harness.rs | Created — SmokeConfig, E2eConfig, RunResult, SmokeRunner, skip macros |
| tests/smoke/fixtures.rs | Created — ResourceTracker, create/delete helpers for Jira + Confluence |
| tests/smoke/validation.rs | Created — 2 validation tests (--version, --help) |
| Cargo.toml | Modified — added `which` + `serde_json` dev-deps, `[[test]]` section for smoke |

## Key Decisions

| Decision | Rationale |
|----------|-----------|
| try_resolve() + resolve() pattern | Skip macros need graceful failure (return None), not panics |
| assert_cmd::Command::new() not cargo_bin | Tests installed binary on PATH, not build output |
| env_remove() for offline mode | Prevents credential leakage from test environment |
| Separate validation.rs module | Matches existing e2e module pattern, keeps main.rs clean |
| E2eConfig duplicated (not shared) | Rust test binaries can't share code without shared crate — pragmatic choice |

## Test Results

- 2 new smoke tests pass (test_version_output, test_help_output)
- 531 total tests pass (452 unit + 70 E2E + 7 integration + 2 smoke)
- Zero clippy warnings on smoke target
- No regressions in existing tests

## Acceptance Criteria

- [x] AC-1: Binary discovery from PATH
- [x] AC-2: Binary discovery from SHRUG_E2E_BINARY
- [x] AC-3: Offline tests run without credentials
- [x] AC-4: Online tests gated by E2E env vars
- [x] AC-5: ResourceTracker works with SmokeRunner
- [x] AC-6: Validation tests pass
- [x] AC-7: Offline tests isolated from environment credentials
