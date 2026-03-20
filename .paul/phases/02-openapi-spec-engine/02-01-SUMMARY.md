---
phase: 02-openapi-spec-engine
plan: 01
subsystem: api
tags: [openapi, parser, serde-json, spec, data-model]

requires:
  - phase: 01-project-foundation
    provides: Error types (ShrugError::SpecError), project scaffold
provides:
  - ApiSpec data model (Operation, Parameter, Tag, HttpMethod, RequestBody)
  - OpenAPI 3.0.1 JSON parser (parse_openapi_v3)
  - Path-level + operation-level parameter merge
affects: [02-02-swagger, 02-03-caching, 03-command-tree]

tech-stack:
  added: []
  patterns: [purpose-built parser (not general OpenAPI lib), parameter merge by name]

key-files:
  created: [src/spec/model.rs, src/spec/parser.rs, src/spec/mod.rs]
  modified: [src/lib.rs]

key-decisions:
  - "Purpose-built data model — only fields needed for CLI generation, no full OpenAPI modeling"
  - "Parameter merge by name — operation-level replaces path-level entirely"
  - "Skip operations without operationId (warn, don't fail)"

patterns-established:
  - "ApiSpec as the universal internal representation for all spec sources"
  - "HttpMethod implements Display as uppercase (GET, POST, etc.)"
  - "No $ref resolution — flat extraction only"

duration: ~10min
started: 2026-03-21T07:05:00Z
completed: 2026-03-21T07:15:00Z
---

# Phase 2 Plan 01: OpenAPI 3.0.1 Parser & Data Model Summary

**Purpose-built OpenAPI 3.0.1 parser extracting operations, parameters, tags, and path templates into shrug's ApiSpec data model for dynamic CLI generation.**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~10min |
| Started | 2026-03-21T07:05:00Z |
| Completed | 2026-03-21T07:15:00Z |
| Tasks | 2 completed |
| Files created/modified | 4 |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: Operations with full metadata | Pass | operationId, method, path, summary, description, tags, deprecated, parameters, request_body |
| AC-2: Tags extracted and deduplicated | Pass | Tag list from spec-level tags array, operations reference by name |
| AC-3: Path templates preserved | Pass | e.g. /rest/api/3/issue/{issueIdOrKey} preserved verbatim |
| AC-4: Real Atlassian spec structure | Pass | Jira-like test fixture: createIssue, getIssue, searchForIssuesUsingJql with pagination params |
| AC-5: Parser errors specific and actionable | Pass | SpecError with context for missing openapi field, version mismatch, invalid JSON |

## Accomplishments

- ApiSpec data model with 7 types: ApiSpec, Operation, Parameter, Tag, HttpMethod, ParameterLocation, RequestBody
- parse_openapi_v3() parser handling paths, operations, parameters, tags, request bodies
- Parameter merge: path-level + operation-level by name, operation wins
- HttpMethod with Display impl (formats as "GET", "POST", etc.)
- 15 unit tests with realistic Jira-like spec fixture

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `src/spec/model.rs` | Created | Data model types with Serialize/Deserialize (80 lines) |
| `src/spec/parser.rs` | Created | Parser + 15 unit tests (480 lines) |
| `src/spec/mod.rs` | Created | Module exports, re-exports parse_openapi_v3 |
| `src/lib.rs` | Modified | Added `pub mod spec` |

## Decisions Made

None beyond plan — executed as written.

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## Next Phase Readiness

**Ready:**
- ApiSpec model ready for Swagger 2.0 conversion target (plan 02-02)
- Parser output ready for caching layer (plan 02-03)
- Operation/Parameter model ready for clap command tree generation (Phase 3)

**Concerns:**
- None

**Blockers:**
- None

---
*Phase: 02-openapi-spec-engine, Plan: 01*
*Completed: 2026-03-21*
