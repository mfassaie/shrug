---
phase: 07-helper-commands-adf
plan: 01
subsystem: cli
tags: [markdown, adf, jql, pulldown-cmark, atlassian]

requires:
  - phase: 06-output-formatting
    provides: ADF terminal renderer (src/adf.rs) used for round-trip validation
provides:
  - Markdown → ADF converter for input bodies
  - JQL shorthand builder for common Jira searches
  - CLI flags: --markdown, --jql, --project, --assignee, --status, --issue-type, --priority, --label
affects: [07-helper-commands-adf, 08-distribution-polish]

tech-stack:
  added: [pulldown-cmark 0.13]
  patterns: [event-to-node-tree stack-based converter, recursive JSON field transformation]

key-files:
  created: [src/markdown_to_adf.rs, src/jql.rs]
  modified: [src/cli.rs, src/main.rs, src/lib.rs, Cargo.toml]

key-decisions:
  - "pulldown-cmark for Markdown parsing — battle-tested, event-based, permissive with malformed input"
  - "Stack-based ADF builder — maps cleanly from pulldown-cmark events to nested ADF node tree"
  - "JQL injection via args, not executor modification — keeps executor generic"
  - "Recursive JSON traversal for --markdown — handles nested fields like {fields:{description:...}}"

patterns-established:
  - "ADF field names constant list (description, body, comment) for Markdown conversion targeting"
  - "JQL value escaping via double-quote escape before wrapping"

duration: ~15min
completed: 2026-03-21
---

# Phase 7 Plan 01: Markdown → ADF Converter and JQL Shorthand Summary

**pulldown-cmark-based Markdown → ADF converter for input bodies, plus JQL shorthand flags (--project, --assignee, --status, etc.) for native Jira search from CLI**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~15 min |
| Completed | 2026-03-21 |
| Tasks | 2 completed |
| Files modified | 6 |
| New tests | 33 (21 markdown + 12 JQL) |
| Total tests | 354 |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: Markdown to ADF conversion (common elements) | Pass | 13 element types covered with 21 unit tests |
| AC-2: Round-trip fidelity | Pass | markdown_to_adf → render_adf preserves headings, lists, code blocks |
| AC-3: --markdown flag converts body | Pass | convert_body_markdown targets description/body/comment at any nesting depth |
| AC-4: JQL shorthand builds correct JQL | Pass | All 6 shorthand fields produce correct clauses, "me" maps to currentUser() |
| AC-5: JQL shorthand combines with raw --jql | Pass | Shorthand clauses ANDed with raw JQL |
| AC-6: JQL shorthand only applies to Jira | Pass | Product::Jira/JiraSoftware check in main.rs, other products ignore flags |

## Accomplishments

- Created `src/markdown_to_adf.rs` (616 lines) — converts Markdown to valid ADF JSON using pulldown-cmark event stream and a stack-based node builder. Supports paragraph, heading, bold, italic, code, lists (nested), code blocks (with language), blockquotes, links, horizontal rules, hard breaks.
- Created `src/jql.rs` (204 lines) — builds JQL from shorthand flags with proper value quoting, double-quote escaping (audit finding), and currentUser() mapping for "me" assignee.
- Wired both features into CLI with 8 new global flags and execution pipeline in main.rs. Markdown conversion runs before request construction, JQL injection appends to args for Jira/JiraSoftware products only.

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `src/markdown_to_adf.rs` | Created | Markdown → ADF converter with pulldown-cmark |
| `src/jql.rs` | Created | JQL shorthand builder with value escaping |
| `src/cli.rs` | Modified | Added 8 new global flags (--markdown, --jql, --project, --assignee, --status, --issue-type, --priority, --label) |
| `src/main.rs` | Modified | Wired markdown conversion and JQL injection into product command path |
| `src/lib.rs` | Modified | Added module declarations for markdown_to_adf and jql |
| `Cargo.toml` | Modified | Added pulldown-cmark 0.13 dependency |

## Decisions Made

| Decision | Rationale | Impact |
|----------|-----------|--------|
| pulldown-cmark 0.13 (not 0.12 as plan specified) | 0.13.1 was latest stable at build time | No impact, API compatible |
| Removed unused Mark::Code variant | Inline code handled via add_inline_code(), not mark stack | Cleaner code, no dead code warning |
| cargo fmt applied to all files | Pre-existing formatting drifts in other files | Clean fmt --check going forward |

## Deviations from Plan

### Summary

| Type | Count | Impact |
|------|-------|--------|
| Auto-fixed | 2 | Minor, no scope change |
| Deferred | 0 | — |

**Total impact:** Minimal — version bump and dead code cleanup.

### Auto-fixed Issues

**1. pulldown-cmark version 0.13 instead of 0.12**
- **Found during:** Task 1 (dependency addition)
- **Issue:** Plan specified 0.12, but 0.13.1 was the latest stable
- **Fix:** Used pulldown-cmark = "0.13" (latest stable)
- **Verification:** All tests pass, API compatible

**2. Dead code warnings (Mark::Code, Heading level_num)**
- **Found during:** Task 1 verification
- **Issue:** Unused enum variant and variable from initial implementation
- **Fix:** Removed Mark::Code (inline code uses separate path), removed unused level_num variable
- **Verification:** clippy clean with -D warnings

## Issues Encountered

None.

## Next Phase Readiness

**Ready:**
- Markdown → ADF converter available for Phase 7 Plan 02 (helper commands like +create will use it)
- JQL shorthand available for +search helper command
- ADF now has both directions: input (markdown_to_adf) and output (adf renderer)

**Concerns:**
- JQL shorthand on non-search Jira operations will produce an "Unknown parameter" error (deferred audit finding, self-correcting)

**Blockers:**
- None

---
*Phase: 07-helper-commands-adf, Plan: 01*
*Completed: 2026-03-21*
