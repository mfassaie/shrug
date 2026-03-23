# Project State

## Project Reference

See: .paul/PROJECT.md (updated 2026-03-23)

**Core value:** Users and AI agents can interact with Atlassian Cloud products from the command line without context-switching to a browser.
**Current focus:** v0.5 MCP Server & Schema Introspection

## Current Position

Milestone: v0.5 MCP Server & Schema Introspection (v0.5.0)
Phase: 21 of 22 (MCP Server)
Plan: Not started
Status: Ready to plan
Last activity: 2026-03-23 — v0.5 milestone created

Progress:
- v0.5 MCP Server & Schema Introspection: [░░░░░░░░░░] 0%
- Phase 21: [░░░░░░░░░░] 0%

## Loop Position

Current loop state:
```
PLAN ──▶ APPLY ──▶ UNIFY
  ○        ○        ○     [Ready for first PLAN]
```

## Accumulated Context

### Decisions
| Decision | Phase | Impact |
|----------|-------|--------|
| BitBucket and JSM excluded from roadmap | v0.3 | Focus on Jira, Jira Software, Confluence |
| Phase 18: rkyv binary cache + ETag serve-stale | Phase 18 | Binary-preferred load, background refresh, conditional fetch |
| Phase 19: confluence +create helper | Phase 19 | Markdown → storage format, product-routed dispatch |
| Phase 20: dynamic completions | Phase 20 | Hidden _complete subcommand, file-cached, all 4 shells |

### Deferred Issues
None.

### Git State
Last commit: da7054f
Branch: main
Tags: v0.1.0, v0.2.0, v0.3.0

## Session Continuity

Last session: 2026-03-23
Stopped at: v0.5 milestone created, ready to plan
Next action: /paul:plan for Phase 21
Resume file: .paul/ROADMAP.md
Resume context:
- v0.5 milestone created: MCP Server + Schema Introspection
- Phase 21: MCP Server (stdio JSON-RPC, full + compact modes)
- Phase 22: Schema Introspection (shrug schema command)
- 529 tests pass, zero clippy warnings

---
*STATE.md — Updated after every significant action*
