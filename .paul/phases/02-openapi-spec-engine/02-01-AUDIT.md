# Enterprise Plan Audit Report

**Plan:** .paul/phases/02-openapi-spec-engine/02-01-PLAN.md
**Audited:** 2026-03-21
**Verdict:** Enterprise-ready after fixes applied

---

## 1. Executive Verdict

**Enterprise-ready after applying 2 must-have + 1 strongly-recommended fixes.**

The plan demonstrates strong architectural judgment: purpose-built data model (not a general OpenAPI library), clear scope boundaries (no $ref, no caching, no downloading), and realistic test expectations (Jira-like excerpts). The primary gap was underspecified parameter merge semantics — a subtle bug source in real OpenAPI specs where path-level and operation-level parameters coexist.

## 2. What Is Solid

- **Purpose-built data model** — Only extracts what CLI generation needs. Avoids the trap of building a full OpenAPI library. This is the correct architectural choice.
- **SpecError for parser failures** — Reuses existing error infrastructure rather than introducing new error types.
- **Graceful handling of missing operationId** — Skip + warn rather than fail. Real specs have operations without operationIds (e.g., OPTIONS for CORS) and hard-failing would break parsing of valid specs.
- **Scope boundaries are excellent** — No $ref resolution, no response schemas, no downloading, no caching. Each is explicitly deferred to the right plan.
- **RequestBody model captures content types** — Critical for knowing whether to send JSON vs form data in Phase 5.
- **Test strategy includes realistic Jira-like excerpts** — Not just synthetic tests.

## 3. Enterprise Gaps Identified

1. **Parameter merge semantics underspecified** — "merge, operation wins" doesn't define the merge key or replacement strategy. OpenAPI 3.0 spec says parameters are identified by `name` + `in` combination, and operation-level overrides path-level. If both path and operation define `maxResults` as query param, the operation's definition should completely replace the path's. Without this, duplicate parameters could appear.

2. **No test for parameter merge behavior** — The most common source of bugs in OpenAPI parsers is incorrect parameter merging. Atlassian specs use path-level parameters extensively (e.g., `issueIdOrKey` defined at path level, shared across GET/PUT/DELETE).

3. **HttpMethod lacks Display impl** — When this model is used in debug logging, error messages, or HTTP request construction (Phase 5), HttpMethod needs to format as "GET", not `Get`. Adding Display now avoids a retrofit.

## 4. Upgrades Applied to Plan

### Must-Have (Release-Blocking)

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | Parameter merge semantics underspecified | Task 2 action (paths walk) | Added: explicit merge-by-name semantics, operation replaces path entirely |
| 2 | No test for parameter merge | Task 2 tests | Added: test for path-level + operation-level merge, operation count verification |

### Strongly Recommended

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | HttpMethod lacks Display impl | Task 1 model definition | Added: HttpMethod must implement Display for "GET"/"POST" formatting |

### Deferred (Can Safely Defer)

| # | Finding | Rationale for Deferral |
|---|---------|----------------------|
| 1 | $ref resolution for parameter refs | Plan explicitly defers this; inline parameters cover Atlassian's primary specs |
| 2 | Enum/oneOf schema types in parameters | Not needed for CLI flag generation; string type hint is sufficient |

## 5. Audit & Compliance Readiness

- **Audit evidence:** Parser tests with Jira-like excerpts provide evidence that the parser handles real-world data.
- **Silent failure prevention:** Missing operationId is logged (warning), not silently dropped. Parser errors include specific field context.
- **Post-incident reconstruction:** Debug-level logging from Phase 1 will show parsed operation counts at startup.
- **Ownership:** Parser is self-contained in src/spec/ with clear public API (parse_openapi_v3).

## 6. Final Release Bar

**What must be true:**
- Parameter merge correctly deduplicates by name, operation-level wins
- HttpMethod formats as uppercase HTTP verbs
- Test coverage includes merge behavior

**Remaining risks:** Minimal. The parser is read-only (no side effects), well-scoped, and the test strategy covers realistic inputs.

**Sign-off:** With the 3 applied fixes, I would sign my name to this system.

---

**Summary:** Applied 2 must-have + 1 strongly-recommended upgrades. Deferred 2 items.
**Plan status:** Updated and ready for APPLY

---
*Audit performed by PAUL Enterprise Audit Workflow*
*Audit template version: 1.0*
