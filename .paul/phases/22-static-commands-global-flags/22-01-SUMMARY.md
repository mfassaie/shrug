# Plan 22-01 Summary

**Phase:** 22 — Static Commands & Global Flags
**Plan:** 01
**Status:** Complete
**Date:** 2026-03-23

## What Was Built

25 offline smoke tests exercising every static CLI command and global flag combination against the installed shrug.exe binary. No Atlassian credentials required.

## Files Created/Modified

| File | Action |
|------|--------|
| tests/smoke/static_commands.rs | Created — 12 tests: profile CRUD, auth, cache, completions |
| tests/smoke/global_flags.rs | Created — 13 tests: output formats, color, verbose, trace, misc flags |
| tests/smoke/main.rs | Modified — added module declarations |

## Test Coverage

**Static commands (12 tests):**
- Profile: create_and_list, show_details, use_sets_default, delete_removes
- Auth: help, status_with_profile
- Cache: help
- Completions: bash, zsh, fish, powershell

**Global flags (13 tests):**
- Output: json, table, yaml, csv, plain
- Color: auto, always, never
- Verbose: -v, -vv, --trace
- Other: --no-pager, --dry-run, --fields

## Key Decisions

| Decision | Rationale |
|----------|-----------|
| Module-scoped unique_name prefix (sc-, gf-) | Prevents collisions between test modules using same process ID |
| test_verbose_v checks exit code only, not stderr | Single -v (INFO level) may not produce output for profile list |
| auth_status_with_profile instead of no-profile variant | More reliable — doesn't depend on developer's existing profile state |
| --fields test accepts exit 0 or field-related error | Flag may not be supported for all commands, but shouldn't crash |

## Test Results

- 25 new smoke tests pass
- 556 total tests pass (452 unit + 70 E2E + 7 integration + 27 smoke)
- Zero clippy warnings
- No regressions
