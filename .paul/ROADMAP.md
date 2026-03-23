# Roadmap: shrug

## Overview

A dynamic CLI for Atlassian Cloud — commands generated at runtime from OpenAPI specs, supporting Jira, Jira Software, and Confluence. Built in Rust, inspired by Google Workspace CLI architecture.

## Current Milestone

**v0.5 MCP Server & Schema Introspection** (v0.5.0)
Status: In Progress
Phases: 0 of 2 complete

## Phases

| Phase | Name | Plans | Status | Completed |
|-------|------|-------|--------|-----------|
| 21 | MCP Server | TBD | Not started | - |
| 22 | Schema Introspection | TBD | Not started | - |

## Phase Details

### Phase 21: MCP Server

**Focus:** Expose Atlassian Cloud APIs as MCP tools for AI agents via stdio JSON-RPC protocol
**Plans:** TBD

**Scope:**
- `shrug mcp` command starts MCP server on stdio
- JSON-RPC 2.0 protocol (initialize, tools/list, tools/call)
- Tool definitions generated dynamically from OpenAPI specs
- Full mode: one tool per API operation (~1,250 tools across all products)
- Compact mode: one tool per product (5 tools with natural language operation routing)
- Reuse existing executor, auth, and spec infrastructure

**Constraints:**
- stdio transport only (no HTTP/SSE in v0.5)
- Must follow MCP specification for tool discovery and execution
- Tool execution must use existing auth profiles
- Must handle large tool lists efficiently

### Phase 22: Schema Introspection

**Focus:** `shrug schema` command for inspecting API operation schemas
**Plans:** TBD

**Scope:**
- `shrug schema <product>` — list available resources/tags
- `shrug schema <product> <resource>` — list operations in a resource
- `shrug schema <product> <resource> <operation>` — show parameters, types, required fields
- Output in table format (TTY) or JSON (pipe)

**Constraints:**
- Must work with both OpenAPI 3.0.1 and Swagger 2.0 specs
- Reuse existing spec parsing infrastructure

## Constraints

- All existing tests must continue to pass (529 unit + doc + integration)
- No regressions in existing CLI behaviour
- MCP protocol compliance is mandatory

## Completed Milestones

<details>
<summary>v0.4 Performance & UX Polish - 2026-03-23 (3 phases, 4 plans)</summary>

| Phase | Name | Plans | Completed |
|-------|------|-------|-----------|
| 18 | Spec Performance | 2 | 2026-03-23 |
| 19 | Confluence Helper | 1 | 2026-03-23 |
| 20 | Dynamic Completions | 1 | 2026-03-23 |

Full archive: `.paul/milestones/v0.4.0-ROADMAP.md`

</details>

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
*Last updated: 2026-03-23 after v0.5 milestone creation*
