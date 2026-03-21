# Enterprise Plan Audit Report

**Plan:** .paul/phases/07-helper-commands-adf/07-01-PLAN.md
**Audited:** 2026-03-21
**Verdict:** Enterprise-ready after fixes applied

---

## 1. Executive Verdict

Conditionally acceptable, upgraded to enterprise-ready after applying findings below.

The plan is well-structured with two focused tasks, clear acceptance criteria in BDD format, and sensible boundaries protecting stable modules. The architectural choice to use pulldown-cmark (established Markdown parser) rather than a hand-rolled parser is correct. JQL injection via args rather than executor modification minimises blast radius.

Two gaps required fixing: JQL value escaping (a correctness and injection-safety issue) and nested JSON field traversal for the --markdown body converter (Jira's API nests description under `fields`). Both have been applied.

I would approve this plan for production after the applied fixes.

## 2. What Is Solid

- **pulldown-cmark as Markdown parser**: Battle-tested, permissive with malformed input, event-based API maps cleanly to ADF node construction. Correct tool choice.
- **Pure function design**: Both `markdown_to_adf()` and `JqlShorthand::build_jql()` are pure functions with no side effects, making them trivially testable and safe.
- **Boundary protection**: Explicitly protects executor, ADF renderer, auth, spec, and command modules. New code is additive only.
- **Round-trip validation strategy**: Testing Markdown → ADF → render_adf gives confidence the ADF structure is valid without needing an Atlassian API call.
- **JQL injection into args, not executor**: Keeps the executor generic. The dynamic CLI's existing parameter validation catches invalid parameter names, providing a natural guard.
- **AC-6 (non-Jira products)**: Correctly scoped. JQL is Jira-only; silently ignoring shorthand for Confluence/Bitbucket is the right behaviour.

## 3. Enterprise Gaps Identified

**Gap 1 — JQL value escaping (severity: must-have)**
The plan specified quoting values with double quotes (`field = "value"`) but did not address values that themselves contain double quotes. A user passing `--status 'Done "Final"'` would produce `status = "Done "Final""` — malformed JQL that would return a 400 error from the Jira API. In a scripting/CI context, this could cause silent pipeline failures.

**Gap 2 — Nested JSON field traversal for --markdown (severity: strongly-recommended)**
The plan targeted fields named "description", "body", "comment" but described walking "all string values" at one point and "known ADF fields" at another. Jira's create/update endpoints nest these under `{"fields": {"description": "..."}}`. A top-level-only scan would miss the most common usage pattern, making --markdown effectively broken for the primary use case.

**Gap 3 — --markdown without --json silent no-op (severity: strongly-recommended)**
If a user passes `--markdown` without `--json`, nothing happens. No error, no warning. In CI/CD pipelines, this silent no-op could mask misconfiguration. A debug/warning log helps users diagnose the issue.

**Gap 4 — JQL shorthand on non-search Jira operations (severity: can-safely-defer)**
If a user passes `--project FOO` with `shrug jira issues get --issue-key FOO-1`, the injected `--jql` arg would hit `parse_args` and produce "Unknown parameter '--jql'". The error message is clear and self-correcting, but could confuse users who didn't expect shorthand flags to produce an error on non-search operations.

## 4. Upgrades Applied to Plan

### Must-Have (Release-Blocking)

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | JQL value escaping — embedded double quotes must be escaped before wrapping in quotes | Task 2 action (build_jql), Task 2 tests | Added explicit escaping instruction and test case for embedded double quotes |

### Strongly Recommended

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | Nested JSON traversal for convert_body_markdown — must handle `{"fields":{"description":"..."}}` | Task 1 action (convert_body_markdown), Task 1 tests | Changed to recursive walk at any nesting depth; added nested field test and skip-objects test |
| 2 | --markdown without --json warning | Task 2 action (main.rs wiring) | Added tracing::warn when --markdown is true but --json is None |

### Deferred (Can Safely Defer)

| # | Finding | Rationale for Deferral |
|---|---------|----------------------|
| 1 | JQL shorthand on non-search Jira operations produces "Unknown parameter" error | Error message from parse_args is clear and self-correcting. Users learn quickly. A future improvement could add help text noting these flags are for search operations. |

## 5. Audit & Compliance Readiness

- **Audit evidence**: Pure functions with comprehensive unit tests produce clear pass/fail evidence. Test coverage spans all supported Markdown elements and JQL patterns.
- **Silent failure prevention**: The --markdown warning (applied) and explicit JQL escaping (applied) prevent the two most likely silent failure modes.
- **Post-incident reconstruction**: JQL shorthand is assembled client-side before the HTTP request, so `--dry-run` will show the final assembled request including JQL. This supports debugging.
- **Ownership**: Both modules are self-contained with clear API boundaries. No cross-module coupling introduced.

## 6. Final Release Bar

**What must be true before shipping:**
- JQL build_jql() must escape embedded double quotes in all value positions
- convert_body_markdown must find target fields at any nesting depth, not just top-level
- All 321 existing tests continue to pass alongside new tests

**Remaining risks if shipped as-is (after fixes):**
- JQL shorthand on non-search operations gives a confusing (but correct) error. Low severity.
- Markdown features not supported (tables, images) will be silently dropped by pulldown-cmark event skipping. Acceptable for v0.1.

I would sign my name to this plan with the applied fixes.

---

**Summary:** Applied 1 must-have + 2 strongly-recommended upgrades. Deferred 1 item.
**Plan status:** Updated and ready for APPLY

---
*Audit performed by PAUL Enterprise Audit Workflow*
*Audit template version: 1.0*
