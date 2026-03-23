# Roadmap: shrug

## Overview

A dynamic CLI for Atlassian Cloud — commands generated at runtime from OpenAPI specs, supporting Jira, Jira Software, and Confluence. Built in Rust, inspired by Google Workspace CLI architecture.

## Current Milestone

**v0.4 Performance & UX Polish** (v0.4.0)
Status: Complete
Phases: 3 of 3 complete

## Phases

| Phase | Name | Plans | Status | Completed |
|-------|------|-------|--------|-----------|
| 18 | Spec Performance | 2 | Complete | 2026-03-23 |
| 19 | Confluence Helper | 1 | Complete | 2026-03-23 |
| 20 | Dynamic Completions | 1 | Complete | 2026-03-23 |

## Phase Details

### Phase 18: Spec Performance

**Focus:** Replace JSON spec cache with rkyv binary cache, add lazy per-product loading, serve-stale with background ETag refresh, and connection pooling
**Plans:** TBD

**Scope:**
- rkyv zero-copy deserialisation for spec cache (<0.01ms warm startup target)
- Lazy per-product spec loading (only load spec for the product being used)
- Serve-stale cache: return expired cache immediately, refresh in background via ETag
- Connection pooling: share single reqwest::Client across all requests

**Constraints:**
- rkyv must handle all fields in src/spec/model.rs
- Lazy loading must not break two-phase CLI parsing
- Rate limits must still be respected with connection reuse

### Phase 19: Confluence Helper

**Focus:** `confluence +create` helper command for creating pages from Markdown files
**Plans:** TBD

**Scope:**
- `shrug confluence +create --space KEY --title "Page Title" --file page.md`
- Markdown → Confluence storage format conversion
- Page creation via Confluence v2 API
- Mirror the UX pattern of `jira +create`

**Constraints:**
- Confluence v2 API requires storage format body wrapper (not ADF)
- Existing markdown_to_adf.rs handles ADF; need storage format converter or adapter

### Phase 20: Dynamic Completions

**Focus:** Tab-completion with real project keys, issue keys, space keys from live Atlassian instance
**Plans:** TBD

**Scope:**
- Dynamic completions for --project, --issueIdOrKey, --space parameters
- Cached lookups with short TTL (avoid hammering API on repeated tab presses)
- Support bash, zsh, fish, PowerShell (extend existing clap_complete setup)

**Constraints:**
- Must work across all 4 shells
- Cache must be fast enough for interactive tab-completion (<100ms)
- Needs auth context to fetch from API

## Constraints

- All existing tests must continue to pass (70 E2E + unit + integration)
- No regressions in existing CLI behaviour
- Performance improvements must be measurable with benchmarks

## Completed Milestones

<details>
<summary>v0.3 Test Coverage & Entity Expansion - 2026-03-23 (5 phases, 9 plans)</summary>

| Phase | Name | Plans | Completed |
|-------|------|-------|-----------|
| 13 | Unit Test Gaps + Bug Fixes + Clippy | 1 | 2026-03-23 |
| 14 | Jira Platform Top 20 | 2 | 2026-03-23 |
| 15 | Jira Software Full Coverage | 2 | 2026-03-23 |
| 16 | Confluence Top 20 | 3 | 2026-03-23 |
| 17 | E2E Feature Gaps | 1 | 2026-03-23 |

Full archive: `.paul/milestones/v0.3.0-ROADMAP.md`

</details>

<details>
<summary>v0.2 E2E Validation - 2026-03-23 (4 phases, 8 plans)</summary>

| Phase | Name | Plans | Completed |
|-------|------|-------|-----------|
| 9 | Test Infrastructure & Auth | 3 | 2026-03-23 |
| 10 | Jira CRUD Tests | 2 | 2026-03-23 |
| 11 | Confluence CRUD Tests | 1 | 2026-03-23 |
| 12 | CLI Feature Tests | 1 | 2026-03-23 |

Full archive: `.paul/milestones/v0.2.0-ROADMAP.md`

</details>

<details>
<summary>v0.1 MVP - 2026-03-21 (8 phases, 24 plans)</summary>

| Phase | Name | Plans | Completed |
|-------|------|-------|-----------|
| 1 | Project Foundation | 3 | 2026-03-21 |
| 2 | OpenAPI Spec Engine | 4 | 2026-03-21 |
| 3 | Dynamic Command Tree | 2 | 2026-03-21 |
| 4 | Authentication & Profiles | 3 | 2026-03-21 |
| 5 | Generic HTTP Executor | 4 | 2026-03-21 |
| 6 | Output & Formatting | 2 | 2026-03-21 |
| 7 | Helper Commands & ADF | 3 | 2026-03-21 |
| 8 | Distribution & Polish | 3 | 2026-03-21 |

Full archive: `.paul/milestones/v0.1.0-ROADMAP.md`

</details>

---
*Roadmap created: 2026-03-21*
*Last updated: 2026-03-23 after v0.4 milestone creation*
