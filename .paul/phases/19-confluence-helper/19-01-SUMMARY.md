---
phase: 19-confluence-helper
plan: 01
subsystem: helpers
tags: [confluence, markdown, storage-format, helper-command, xhtml]

requires:
  - phase: 07-helpers
    provides: helper dispatch pattern, parse_helper_args, send_json_request
provides:
  - Markdown to Confluence storage format converter
  - confluence +create helper command
  - Space key to ID resolution
affects: []

tech-stack:
  added: []
  patterns: [product-routed helper dispatch, Confluence storage format XHTML]

key-files:
  created: [src/markdown_to_storage.rs]
  modified: [src/helpers.rs, src/lib.rs]

key-decisions:
  - "Dispatch routes by product first, then by helper name. Cleaner than one big match"
  - "Space key resolved to ID via GET /wiki/api/v2/spaces?keys= (v2 API uses numeric IDs)"
  - "Storage format used instead of ADF for Confluence page bodies"

patterns-established:
  - "Product-specific helper routing: Confluence and Jira have separate helper sets"
  - "Markdown to storage format: pulldown-cmark events to XHTML with Confluence macros"

duration: ~15min
started: 2026-03-23T00:00:00Z
completed: 2026-03-23T00:00:00Z
---

# Phase 19 Plan 01: Confluence +create Helper Summary

**Confluence +create helper with Markdown to storage format conversion and space key resolution.**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~15min |
| Tasks | 2 planned, 2 executed |
| Files created | 1 (markdown_to_storage.rs) |
| Files modified | 2 (helpers.rs, lib.rs) |
| Tests added | 16 new tests (13 converter + 3 helper) |
| Total tests | 518 (441 unit + 70 doc + 7 integration), 1 ignored |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: Markdown to storage format | Pass | 13 unit tests covering headings, bold, italic, lists, code blocks, links, blockquotes |
| AC-2: confluence +create creates a page | Pass | Builds correct POST body with storage representation |
| AC-3: File input reads Markdown from disk | Pass | --file reads and converts file contents |
| AC-4: Helper dispatch extended for Confluence | Pass | Product-routed dispatch, unknown helpers list Confluence-specific options |

## Accomplishments

- `src/markdown_to_storage.rs`: Full Markdown to Confluence storage format converter (XHTML)
  - Handles paragraphs, headings (h1-h6), bold, italic, strikethrough, code spans
  - Code blocks use `ac:structured-macro` with optional language parameter
  - Lists, links, blockquotes, horizontal rules, images, hard breaks
  - XML escaping for safe output
- `helper_confluence_create`: accepts --space, --title, --body/--file, --parent-id
- `resolve_space_id`: translates space key → numeric space ID via v2 API
- `dispatch_helper` refactored: routes by product first, supports Confluence + Jira separately
- 16 new unit tests

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `src/markdown_to_storage.rs` | Created | Markdown → Confluence storage format (XHTML) converter |
| `src/helpers.rs` | Modified | Product-routed dispatch, helper_confluence_create, resolve_space_id |
| `src/lib.rs` | Modified | Added `pub mod markdown_to_storage` |

## Decisions Made

| Decision | Rationale | Impact |
|----------|-----------|--------|
| Storage format (not ADF) for Confluence | Confluence v2 API uses storage representation for page bodies | New converter needed, can't reuse markdown_to_adf |
| Product-first routing in dispatch | Cleaner separation, each product has its own helper list | Easy to add more Confluence helpers later |
| Space key → ID resolution via API | v2 API requires numeric space IDs, users think in space keys | Extra API call on create, but user-friendly |

## Deviations from Plan

None.

## Issues Encountered

None.

## Next Phase Readiness

**Ready:**
- Phase 19 complete (single plan)
- Ready for Phase 20 (Dynamic Completions)

**Concerns:**
- None

**Blockers:**
- None

---
*Phase: 19-confluence-helper, Plan: 01*
*Completed: 2026-03-23*
