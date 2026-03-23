# Project State

## Project Reference

See: .paul/PROJECT.md (updated 2026-03-23)

**Core value:** Users and AI agents can interact with Atlassian Cloud products from the command line without context-switching to a browser.
**Current focus:** v0.6 MCP Server & Schema Introspection

## Current Position

Milestone: Awaiting next milestone
Phase: None active
Plan: None
Status: Milestone v0.5 Windows E2E Smoke Tests complete — ready for next
Last activity: 2026-03-23 — Milestone completed

Progress:
- v0.5 Windows E2E Smoke Tests: [██████████] 100% ✓

## Loop Position

Current loop state:
```
PLAN ──▶ APPLY ──▶ UNIFY
  ○        ○        ○     [Milestone complete - ready for next]
```

## Accumulated Context

### Decisions
| Decision | Phase | Impact |
|----------|-------|--------|
| BitBucket and JSM excluded from roadmap | v0.3 | Focus on Jira, Jira Software, Confluence |
| Phase 18: rkyv binary cache + ETag serve-stale | Phase 18 | Binary-preferred load, background refresh, conditional fetch |
| Phase 19: confluence +create helper | Phase 19 | Markdown → storage format, product-routed dispatch |
| Phase 20: dynamic completions | Phase 20 | Hidden _complete subcommand, file-cached, all 4 shells |
| MCP Server milestone deferred to v0.6 | v0.5 | Windows E2E smoke tests prioritised |
| try_resolve() pattern for binary discovery | Phase 21 | Skip macros use graceful failure, not panics |
| insta for help message snapshots | Phase 23 | Golden-file regression testing |

### Deferred Issues
None.

### Git State
Last commit: da7054f
Branch: main
Tags: v0.1.0, v0.2.0, v0.3.0, v0.5.0

## Session Continuity

Last session: 2026-03-23
Stopped at: Milestone v0.5 complete
Next action: /paul:discuss-milestone or /paul:milestone for v0.6
Resume file: .paul/MILESTONES.md
Resume context:
- v0.5 complete: 47 smoke tests, 576 total tests pass
- v0.6 MCP Server & Schema Introspection ready (phases 25-26)

---
*STATE.md — Updated after every significant action*
