---
phase: 06-output-formatting
plan: 02
subsystem: output
tags: [fields, pager, adf, atlassian-document-format, column-selection]

requires:
  - phase: 06-output-formatting
    provides: format_response(), format_table(), format_csv(), output module
provides:
  - --fields column selection for table/CSV output
  - Pager integration ($PAGER / less -R -F -X)
  - --no-pager flag
  - ADF terminal renderer (paragraph, heading, list, code, marks)
  - ADF detection and rendering in table cells and plain text
affects: [07-helper-commands]

tech-stack:
  added: []
  patterns: [filter_fields for column selection, print_with_pager, ADF recursive node walker]

key-files:
  created: [src/adf.rs]
  modified: [src/output.rs, src/cli.rs, src/executor.rs, src/main.rs, src/lib.rs, Cargo.toml]

key-decisions:
  - "serde_json preserve_order feature enabled for --fields ordering"
  - "Pager disabled for paginated output (audit-added: incompatible with print-as-you-go)"
  - "Pager default: less -R -F -X (audit-added: -F quits if fits, -X no screen clear)"
  - "ADF unknown node types silently skipped for forward compatibility"

patterns-established:
  - "filter_fields() extracts from wrapper objects before filtering"
  - "print_with_pager() wraps output with graceful pager fallback"
  - "adf::is_adf() + adf::render_adf() for inline ADF detection and rendering"

duration: ~15min
completed: 2026-03-21
---

# Phase 6 Plan 02: ADF Rendering, Pager, and --fields Summary

**--fields column selection, pager integration with $PAGER support, and ADF terminal renderer for Atlassian Document Format content in table cells and plain text output.**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~15min |
| Completed | 2026-03-21 |
| Tasks | 2 completed |
| Files modified | 7 (1 created, 6 modified) |
| Tests added | 22 (321 total) |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: --fields filters table columns | Pass | filter_fields retains specified keys in user-specified order |
| AC-2: --fields filters CSV columns | Pass | format_csv_with_fields respects --fields order, overrides alphabetical |
| AC-3: --fields ignored for JSON/YAML/plain | Pass | Only applied when format is Table or Csv |
| AC-4: Pager activated for long TTY output | Pass | print_with_pager pipes through $PAGER or less -R -F -X |
| AC-5: --no-pager disables pager | Pass | Passed through executor to print_with_pager |
| AC-6: ADF rendered as terminal text | Pass | 13 node types supported, ANSI marks with colour toggle |
| AC-7: ADF in JSON/YAML left unchanged | Pass | ADF rendering only in truncate_value and format_plain |

## Accomplishments

- Created src/adf.rs with recursive node walker for 13 ADF node types
- Added --fields and --no-pager CLI flags
- Created filter_fields() with wrapper object extraction
- Integrated pager with graceful fallback on spawn failure
- Enabled serde_json preserve_order for deterministic field ordering
- ADF automatically detected and rendered in table cells and plain text output

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `src/adf.rs` | Created | ADF terminal renderer with 13 node types, 14 tests |
| `src/output.rs` | Modified | filter_fields, format_csv_with_fields, print_with_pager, ADF integration, 8 new tests |
| `src/cli.rs` | Modified | --fields and --no-pager flags |
| `src/executor.rs` | Modified | fields and no_pager params, print_with_pager for single requests |
| `src/main.rs` | Modified | Parse fields, pass no_pager through |
| `src/lib.rs` | Modified | Added pub mod adf |
| `Cargo.toml` | Modified | serde_json preserve_order feature |

## Deviations from Plan

None. Plan executed exactly as written (including audit-added items).

## Next Phase Readiness

**Ready:**
- Phase 6 is complete (2/2 plans). Full output formatting pipeline built.
- Phase 7 (Helper Commands & ADF) can build on the output formatters and ADF renderer.
- Markdown → ADF conversion (07-01) is the input-side complement to this output-side ADF renderer.

**Concerns:**
- ADF renderer covers common nodes only. Media, tables, panels, expand nodes are silently skipped.

**Blockers:** None

---
*Phase: 06-output-formatting, Plan: 02*
*Completed: 2026-03-21*
