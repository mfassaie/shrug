# Project State

## Project Reference

See: .paul/PROJECT.md (updated 2026-03-23)

**Core value:** Users and AI agents can interact with Atlassian Cloud products from the command line without context-switching to a browser.
**Current focus:** v0.4 Performance & UX Polish — Spec Performance first

## Current Position

Milestone: v0.4 Performance & UX Polish (v0.4.0)
Phase: 18 of 20 (Spec Performance)
Plan: 18-01 applied (rkyv binary cache)
Status: APPLY complete, ready for UNIFY
Last activity: 2026-03-23 — Applied 18-01: rkyv binary spec cache with JSON fallback

Progress:
- v0.4 Performance & UX Polish: [██░░░░░░░░] 17%
- Phase 18: [█████░░░░░] 50%

## Loop Position

Current loop state:
```
PLAN ──▶ APPLY ──▶ UNIFY
  ✓        ✓        ○     [Applied, ready for UNIFY]
```

## Accumulated Context

### Decisions
| Decision | Phase | Impact |
|----------|-------|--------|
| BitBucket and JSM excluded from roadmap | v0.3 | Focus on Jira, Jira Software, Confluence |
| +search updated to enhanced search API | Phase 13 | Uses search-and-reconsile-issues-using-jql |
| Global shorthand flags forwarded to helpers | Phase 13 | --project, --assignee, --status reach +create etc. |
| Phase 14: 6 new Jira E2E entity tests, 20 entities total | Phase 14 | Attachment multipart fallback pattern |
| Phase 15: 6 JSW tests, jira_software.rs module | Phase 15 | Board CRUD requires filter dependency |
| Phase 16: 15 new Confluence tests, 20 entities total | Phase 16 | get_space_id/create_page helpers |
| Phase 17: 4 feature tests (pagination, verbose, trace, ADF) | Phase 17 | All v0.3 E2E gaps closed |

### Deferred Issues
None.

### Git State
Last commit: 35bd7f5
Branch: main
Tags: v0.1.0, v0.2.0, v0.3.0

## Session Continuity

Last session: 2026-03-23
Stopped at: Plan 18-01 APPLY complete, UNIFY pending
Next action: /paul:unify .paul/phases/18-spec-performance/18-01-PLAN.md
Resume file: .paul/HANDOFF-2026-03-23-phase18.md
Resume context:
- Plan 18-01 applied: rkyv binary spec cache (7 model types, dual-write, binary-preferred load)
- 495 tests pass, zero clippy warnings
- Plan 18-02 needed: background ETag refresh
- Connection pooling + lazy loading already implemented (no work needed)

---
*STATE.md — Updated after every significant action*
