# Enterprise Plan Audit Report

**Plan:** .paul/phases/05-generic-http-executor/05-01-PLAN.md
**Audited:** 2026-03-21
**Verdict:** Enterprise-ready after fixes applied

---

## 1. Executive Verdict

Conditionally acceptable, now enterprise-ready after applying fixes. The plan's architecture is sound: it correctly separates argument parsing, request building, and execution into testable units. The dependency chain (ResolvedCommand → ParsedArgs → HTTP request → response mapping) is clean and well-bounded. The three findings applied close genuine gaps that would have caused real production issues (broken URLs for Atlassian's `{baseUrl}` pattern, missing 400 handling, credential leakage in dry-run output).

I would approve this plan for production with the applied fixes.

---

## 2. What Is Solid

**Argument parsing design.** Separating parse_args as a pure function from the HTTP call is correct. It makes the most complex part (flag→parameter matching with kebab/camelCase conversion) independently testable without network. This is the right layering.

**Scope boundaries.** The plan explicitly excludes retry, pagination, and quirks (Plans 05-02, 05-03, 05-04). This prevents scope creep and keeps the first plan focused on the core request lifecycle. The DO NOT CHANGE list correctly protects all prior-phase modules.

**Credential flow.** The plan correctly identifies that `main.rs` currently resolves credentials but doesn't pass them through. Fixing this flow is the right call.

**Connection pooling.** Using a single `reqwest::Client` created once at the top of `run()` is the standard pattern for reqwest. Correct and sufficient.

**Error type mapping.** Using existing `ShrugError` variants avoids introducing new error types. The variant set (AuthError, NotFound, PermissionDenied, RateLimited, ServerError, NetworkError) covers the HTTP status space well.

---

## 3. Enterprise Gaps Identified

### Gap 1: Site URL substitution (MUST-HAVE)
Atlassian specs declare `{baseUrl}` as the server URL variable. The existing `strip_server_variables()` in `analysis.rs` strips this to an empty string. Without substituting the profile's site URL, all constructed URLs would be path-only (e.g., `/rest/api/3/issue/TEST-1`) with no host. Every request would fail.

### Gap 2: HTTP 400 Bad Request not mapped (MUST-HAVE)
The plan mapped 401, 403, 404, 429, 5xx but omitted 400. This is the most common error response from Atlassian APIs (invalid JQL, malformed body, missing required fields). Unmapped 400s would likely fall through as a generic error without the response body, making debugging impossible.

### Gap 3: Dry-run credential leakage (STRONGLY-RECOMMENDED)
The plan specified printing headers to stderr in dry-run mode, but didn't address masking the Authorization header. A user running `--dry-run` and copying the output into a bug report or terminal log would leak their API token or OAuth access token.

### Gap 4: Error response body not included in error messages (STRONGLY-RECOMMENDED)
The plan said to map status codes to ShrugError variants, but didn't specify including the response body text. Atlassian APIs return structured JSON error messages. Without these, users get "Authentication failed" instead of "Authentication failed: API token is invalid for user@example.com".

### Gap 5: 204 No Content handling (STRONGLY-RECOMMENDED)
DELETE operations and some POSTs return 204 with no body. The plan said "print response body to stdout" for 2xx, which would print nothing or potentially error on empty body parsing.

---

## 4. Upgrades Applied to Plan

### Must-Have (Release-Blocking)

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | Site URL substitution for `{baseUrl}` specs | Task 1 action, build_request() | Added site URL substitution logic: if server_url is empty/stripped, use credential's site field as base URL |
| 2 | HTTP 400 Bad Request mapping | AC-4, Task 1 action (execute) | Added 400 → UsageError mapping with response body inclusion |

### Strongly Recommended

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | Dry-run credential masking | AC-5, Task 1 action (execute) | Added requirement to mask Authorization header values in dry-run output |
| 2 | Error response body in error messages | AC-4, Task 1 action (execute) | Added requirement to read and include response body text in all error ShrugError messages |
| 3 | 204 No Content handling | AC-4, Task 1 action (execute) | Added 204 → silent success handling |

### Deferred (Can Safely Defer)

| # | Finding | Rationale for Deferral |
|---|---------|----------------------|
| 1 | Boolean/array parameter handling (flags without values, multi-value params) | Current plan treats all params as `--key value` string pairs. This covers the vast majority of Atlassian API parameters. Array params and boolean flags can be added in a follow-up without architectural changes. |
| 2 | `--json @file.json` file input syntax | Reading body from a file is a UX convenience. The `--json '{...}'` string input covers the functional need. File input can be added later without changing the executor architecture. |
| 3 | Header and Cookie parameter injection | Atlassian APIs rarely use Header/Cookie parameters in their specs. The model supports them but the executor can skip them for now. No spec-defined operation requires them for basic functionality. |

---

## 5. Audit & Compliance Readiness

**Audit evidence:** The dry-run mode (with credential masking) produces reconstructable evidence of what shrug would send. This is sufficient for post-incident analysis.

**Silent failures:** The 204 handling fix prevents the executor from silently producing confusing output on successful DELETE operations. Error response body inclusion prevents silent swallowing of API diagnostic information.

**Post-incident reconstruction:** With the `--trace` logging flag (Phase 1) and the dry-run mode, request details can be reconstructed. The credential masking ensures logs don't contain secrets.

**Ownership:** The executor module has clear ownership (single file, clear API surface). The `execute()` function is the single entry point for all API calls, making it the obvious place for future cross-cutting concerns (rate limiting, retries).

---

## 6. Final Release Bar

**What must be true before this ships:**
- Site URL substitution works correctly for all 5 Atlassian products (some use full URLs, some use `{baseUrl}`)
- 400, 401, 403, 404, 429, 5xx all produce actionable error messages with API response details
- Dry-run output never contains real credentials
- All existing 203 tests continue to pass

**Risks if shipped as-is (before fixes):**
- Every request to specs using `{baseUrl}` (most Atlassian specs) would have produced an invalid URL
- 400 errors would have produced generic messages, making the CLI unusable for debugging

**Sign-off:** With the applied fixes, I would sign my name to this plan. The scope is well-bounded, the architecture is clean, and the deferred items are genuinely safe to defer.

---

**Summary:** Applied 2 must-have + 3 strongly-recommended upgrades. Deferred 3 items.
**Plan status:** Updated and ready for APPLY.

---
*Audit performed by PAUL Enterprise Audit Workflow*
*Audit template version: 1.0*
