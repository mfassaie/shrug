---
phase: 07-helper-commands-adf
plan: 02
subsystem: cli
tags: [helpers, jira, create, search, transition, reqwest]

requires:
  - phase: 07-helper-commands-adf
    provides: Markdown→ADF converter (07-01), JQL shorthand builder (07-01)
provides:
  - Helper commands (+create, +search, +transition) for Jira
  - Helper dispatch infrastructure (extensible for future products)
  - Direct HTTP request utility for helpers
affects: [07-helper-commands-adf, 08-distribution-polish]

tech-stack:
  added: []
  patterns: [helper command bypass routing, direct HTTP with auth injection, two-step API flow for transitions]

key-files:
  created: [src/helpers.rs]
  modified: [src/main.rs, src/lib.rs]

key-decisions:
  - "Direct HTTP via reqwest client instead of executor::execute() — needed for response body access"
  - "Product validation in dispatch_helper — Jira/JiraSoftware only, clear error for others"
  - "OperationId not-found guards on all spec lookups — clear error if spec changes"

patterns-established:
  - "Helper commands use + prefix, intercepted before normal routing"
  - "Helper arg parsing: simple --key value HashMap, require_arg() for validation"
  - "send_json_request() utility for direct authenticated JSON API calls"

duration: ~15min
completed: 2026-03-21
---

# Phase 7 Plan 02: Helper Commands (+create, +search, +transition) Summary

**Three Jira helper commands (+create, +search, +transition) that bypass dynamic routing and make direct authenticated API calls with ergonomic flag-based interfaces**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~15 min |
| Completed | 2026-03-21 |
| Tasks | 2 completed |
| Files modified | 3 |
| New tests | 16 |
| Total tests | 370 |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: +create builds and sends create issue request | Pass | Builds JSON body with project/summary/issuetype, prints created key |
| AC-2: +create description auto-converts Markdown to ADF | Pass | Uses markdown_to_adf::markdown_to_adf() on --description value |
| AC-3: +search executes JQL from shorthand flags | Pass | Reuses JqlShorthand, formats via output::format_response() |
| AC-4: +search defaults to current user's open issues | Pass | Falls back to "assignee = currentUser() AND resolution = Unresolved" |
| AC-5: +transition resolves name and applies | Pass | Two-step: GET transitions, match name (case-insensitive), POST |
| AC-6: +transition error lists available transitions | Pass | Clear error with all available transition names |
| AC-7: Unknown helper produces helpful error | Pass | Lists +create, +search, +transition |

## Accomplishments

- Created `src/helpers.rs` (736 lines) with three Jira helper commands, a helper argument parser, operation lookup with not-found guards, direct HTTP request utility with auth injection, and product validation.
- Wired helpers into main.rs to intercept `+` prefixed commands before normal routing, sharing credential resolution and JQL shorthand from the existing pipeline.
- All helpers support --dry-run, including explicit handling for the multi-step +transition flow.

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `src/helpers.rs` | Created | Helper command dispatcher, +create/+search/+transition, HTTP utilities |
| `src/main.rs` | Modified | Helper interception before handle_product(), spec loading for helpers |
| `src/lib.rs` | Modified | Added helpers module declaration |

## Decisions Made

| Decision | Rationale | Impact |
|----------|-----------|--------|
| Direct HTTP requests via reqwest (not executor::execute()) | executor::execute() formats and prints but doesn't return response body; helpers need body for key extraction and transition parsing | Duplicates auth injection (~15 lines) but avoids modifying executor boundary |
| OperationId not-found guards | Hardcoded operationIds could break if Atlassian renames them; clear error prevents confusing failures | Each of 4 lookups has explicit error message |
| Product validation in dispatch_helper | Helpers are Jira-specific; running on Confluence would produce confusing spec-mismatch errors | Clear "only available for Jira" message |

## Deviations from Plan

### Summary

| Type | Count | Impact |
|------|-------|--------|
| Auto-fixed | 1 | Minor |
| Deferred | 0 | — |

**Total impact:** Minimal.

### Auto-fixed Issues

**1. ShrugError::ServerError struct variant syntax**
- **Found during:** Task 1 compilation
- **Issue:** Plan assumed tuple variant `ServerError(String)`, actual definition is `ServerError { status: u16, message: String }`
- **Fix:** Used struct variant syntax with status code from HTTP response
- **Verification:** All tests pass

## Issues Encountered

None.

## Next Phase Readiness

**Ready:**
- Helper infrastructure extensible for Confluence helpers in future
- send_json_request() utility reusable by any future helper
- Phase 7 plan 03 (shell completions, field/user caches) has all prerequisites met

**Concerns:**
- No retry logic in helpers' direct HTTP calls (executor retries are private). Single-attempt only. Acceptable for interactive commands.
- OperationId values are hardcoded. If Atlassian renames them, helpers fail with clear errors.

**Blockers:**
- None

---
*Phase: 07-helper-commands-adf, Plan: 02*
*Completed: 2026-03-21*
