---
phase: 06-output-formatting
plan: 01
subsystem: output
tags: [formatters, json, table, yaml, csv, plain, tty-detection, no-color]

requires:
  - phase: 05-generic-http-executor
    provides: execute() with println!("{}", body) output, OutputFormat enum in cli.rs
provides:
  - Output formatter module with 5 format renderers
  - TTY auto-detection (table for TTY, JSON for pipes)
  - NO_COLOR support
  - format_response() integrated into executor send pipeline
affects: [06-02-adf-pager-fields, 07-helper-commands]

tech-stack:
  added: [comfy-table 7, serde_yaml 0.9, csv 1, is-terminal 0.4]
  patterns: [format_response returns String, resolve_format for TTY auto-detection]

key-files:
  created: [src/output.rs]
  modified: [src/executor.rs, src/main.rs, src/lib.rs, Cargo.toml]

key-decisions:
  - "Non-JSON body fallback: return raw body unchanged (audit-added)"
  - "CSV columns sorted alphabetically for deterministic output (audit-added)"
  - "comfy-table UTF8_FULL_CONDENSED preset for table rendering"
  - "force_no_tty() when color disabled to suppress ANSI codes"

patterns-established:
  - "format_response() as the single entry point for all output formatting"
  - "resolve_format() handles TTY auto-detection before executor runs"
  - "extract_results_array() shared between table and CSV formatters"

duration: ~10min
completed: 2026-03-21
---

# Phase 6 Plan 01: Output Formatters Summary

**Five output formatters (JSON, table, YAML, CSV, plain) with TTY auto-detection and NO_COLOR support, integrated into the executor's output pipeline.**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~10min |
| Completed | 2026-03-21 |
| Tasks | 2 completed |
| Files modified | 5 (1 created, 4 modified) |
| Tests added | 21 (299 total) |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: JSON pretty-printed | Pass | serde_json::to_string_pretty with 2-space indent |
| AC-2: Table renders key-value and columnar | Pass | comfy-table with Field/Value for objects, column headers for arrays |
| AC-3: YAML valid output | Pass | serde_yaml::to_string |
| AC-4: CSV with headers | Pass | Alphabetically sorted columns, csv crate writer |
| AC-5: Plain text key: value | Pass | Top-level scalars as key: value, nested as compact JSON |
| AC-6: TTY auto-detection | Pass | resolve_format() returns Json when Table + not TTY |
| AC-7: NO_COLOR respected | Pass | should_use_color() checks env, force_no_tty() on table |
| AC-8: Pagination formatted per page | Pass | format_response() called per page in execute_paginated() |

## Accomplishments

- Created src/output.rs with format_response(), 5 format renderers, TTY detection, colour control
- Integrated formatters into executor: replaced both println!("{}", body) calls
- Added non-JSON body fallback (audit finding: Atlassian can return HTML error pages)
- Added deterministic CSV column ordering (audit finding: sorted alphabetically)
- 21 comprehensive tests covering all formats, edge cases, and TTY logic

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `Cargo.toml` | Modified | Added comfy-table 7, serde_yaml 0.9, csv 1, is-terminal 0.4 |
| `src/output.rs` | Created | Output formatters, TTY detection, colour control, 21 tests |
| `src/executor.rs` | Modified | format/is_tty/color_enabled params on execute() and execute_paginated(), format_response() calls |
| `src/main.rs` | Modified | Compute is_tty, resolve_format, should_use_color, pass through to executor |
| `src/lib.rs` | Modified | Added pub mod output |

## Deviations from Plan

None. Plan executed exactly as written (including audit-added items).

## Next Phase Readiness

**Ready:**
- 06-02 (ADF rendering, pager, --fields) can build on format_response() pattern
- Pager integration will wrap the println!("{}", formatted) call
- --fields selection can filter JSON keys before passing to formatter

**Concerns:**
- Table truncation at 60 chars may cut important data for wide responses. Users can use --output json for full output.

**Blockers:** None

---
*Phase: 06-output-formatting, Plan: 01*
*Completed: 2026-03-21*
