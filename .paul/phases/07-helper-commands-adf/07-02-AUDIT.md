# Enterprise Plan Audit Report

**Plan:** .paul/phases/07-helper-commands-adf/07-02-PLAN.md
**Audited:** 2026-03-21
**Verdict:** Enterprise-ready after fixes applied

---

## 1. Executive Verdict

Conditionally acceptable, upgraded to enterprise-ready after applying fixes below.

The plan has a sound architecture: helper commands as a clean bypass layer above the normal routing pipeline, reusing existing infrastructure (JQL shorthand, Markdown converter, executor). The three helpers cover the most valuable Jira UX shortcuts.

Two architectural issues required fixing: (1) the plan specified delegating to `executor::execute()` which doesn't return the response body, making it impossible to parse the created issue key or transition list; and (2) hardcoded operationId lookups had no guard for the case where the operation doesn't exist in the loaded spec. Both have been addressed.

I would approve this plan for production after the applied fixes.

## 2. What Is Solid

- **"+" prefix convention**: Clean, unambiguous, won't collide with any tag or operation name from specs. Easy to detect before routing.
- **Bypass architecture**: Helpers intercept before route_product(), avoiding any coupling with the dynamic command tree. Clean separation.
- **Reuse of 07-01 infrastructure**: +search reuses JqlShorthand directly. +create reuses markdown_to_adf(). No duplication.
- **Two-step +transition pattern**: GET transitions then POST is the correct Jira API flow. Case-insensitive matching is user-friendly.
- **Scope limits are appropriate**: Jira-only, no interactive prompts, no custom fields. Keeps the plan focused.
- **Error listing for unknown helpers**: AC-7 ensures discoverable UX.

## 3. Enterprise Gaps Identified

**Gap 1 — executor::execute() doesn't return response body (severity: must-have)**
The plan specified "Delegate to executor::execute()" for all three helpers, but this function formats and prints the response directly (returns `Result<(), ShrugError>`). Helpers that need to parse the response (all three of them) cannot use this function. +create needs the response body to extract the issue key. +transition needs the GET response to parse available transitions. +search needs the body for formatting.

**Gap 2 — No guard for missing operationId in spec (severity: must-have)**
The plan hardcodes operationId values ("createIssue", "searchForIssuesUsingJql", "getTransitions", "doTransition"). If a spec doesn't contain the expected operationId (spec version change, different product), the lookup would fail. Without an explicit guard, this could produce confusing errors or panics depending on implementation.

**Gap 3 — No product validation in dispatch_helper (severity: strongly-recommended)**
The plan says "Only Jira helpers for now" in scope limits, but dispatch_helper doesn't check the product. Running `shrug confluence +create` would attempt to find "createIssue" in the Confluence spec, fail with a confusing error, and not tell the user that helpers are Jira-only.

**Gap 4 — +transition dry-run makes real GET request (severity: strongly-recommended)**
+transition is a two-step process. In dry-run mode, the first GET request would either execute (side-effect in a mode that shouldn't have side-effects) or return no data (if using executor::execute which short-circuits for dry-run). Either way, the transition name resolution fails. Dry-run for multi-step helpers needs explicit handling.

## 4. Upgrades Applied to Plan

### Must-Have (Release-Blocking)

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | Helpers must make HTTP requests directly via reqwest client, not executor::execute() | Task 1 action (+create, +search, +transition) | Changed all three helpers to make direct HTTP requests with auth header injection, using output::format_response() for formatting |
| 2 | Operation ID not-found guard | Task 1 action (+create, +search, +transition) | Added explicit error message when operationId not found in spec for all four lookups |

### Strongly Recommended

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | Product validation in dispatch_helper | Task 1 action (dispatch_helper) | Added Jira/JiraSoftware check with clear error for other products |
| 2 | +transition dry-run handling | Task 1 action (+transition) | Added: if dry_run, print request URLs and return without making requests |

### Deferred (Can Safely Defer)

| # | Finding | Rationale for Deferral |
|---|---------|----------------------|
| — | None | — |

## 5. Audit & Compliance Readiness

- **Audit evidence**: Unit tests cover all helper logic including error paths. Direct HTTP approach gives full control over request/response cycle.
- **Silent failure prevention**: OperationId guards ensure clear errors when spec doesn't match expectations. Product validation prevents confusing cross-product errors.
- **Post-incident reconstruction**: Helpers use the same credential/auth flow as normal commands. Dry-run support works correctly for single-step helpers and is now explicitly handled for multi-step (+transition).
- **Ownership**: Single module (src/helpers.rs) with clear boundaries. No cross-module coupling introduced.

## 6. Final Release Bar

**What must be true before shipping:**
- All four operationId lookups must have not-found guards
- Helpers make direct HTTP requests (not executor::execute())
- Product validation rejects non-Jira products
- +transition dry-run doesn't attempt real requests

**Remaining risks if shipped as-is (after fixes):**
- OperationId values are hardcoded. If Atlassian renames them, helpers break (with clear error messages). Low probability, acceptable risk.
- No retry logic in helpers' direct HTTP calls (executor's retry logic is in the private execute_with_retry). Single-attempt only. Acceptable for helpers.

I would sign my name to this plan with the applied fixes.

---

**Summary:** Applied 2 must-have + 2 strongly-recommended upgrades. Deferred 0 items.
**Plan status:** Updated and ready for APPLY

---
*Audit performed by PAUL Enterprise Audit Workflow*
*Audit template version: 1.0*
