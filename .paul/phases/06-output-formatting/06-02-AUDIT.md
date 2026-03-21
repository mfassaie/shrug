# Enterprise Plan Audit Report

**Plan:** .paul/phases/06-output-formatting/06-02-PLAN.md
**Audited:** 2026-03-21
**Verdict:** Enterprise-ready after fixes applied

---

## 1. Executive Verdict

Conditionally acceptable, now enterprise-ready after applying 2 findings. The plan covers three distinct features (--fields, pager, ADF) in a well-scoped way. The pager-pagination conflict was a clear correctness issue that would have created a broken experience. The missing pager flags would have annoyed users. Both applied. I would approve this for production.

## 2. What Is Solid

- **ADF renderer scoping.** Supporting common node types and silently skipping unknown ones is exactly right for forward compatibility. Atlassian adds new ADF nodes regularly; crashing on them would be unacceptable.
- **--fields applies only to table/CSV.** Not filtering JSON/YAML output is correct. Users piping JSON expect the full response. Filtering should only affect display formats.
- **filter_fields respects wrapper objects.** Extracting from issues/values/results before filtering means --fields works transparently on Atlassian's envelope pattern.
- **Pager fallback on spawn failure.** Graceful degradation to println if the pager binary is missing.
- **ADF mark rendering with colour toggle.** ANSI codes only when colour enabled prevents mojibake in non-terminal contexts.

## 3. Enterprise Gaps Identified

### Gap 1: Pager per page in pagination (broken UX)
The plan replaces both println calls with print_with_pager. For paginated output (execute_paginated), each page is printed in a loop. Spawning a pager per page would create N separate pager sessions requiring N user interactions to dismiss. This is a UX-breaking bug.

### Gap 2: Incomplete pager flags
The task action specifies "less -R" as the default, but the scope limits section says "less -R -F -X". Without -F (quit if content fits screen), even a one-line response opens the pager. Without -X, less clears the screen on exit and users lose the output.

## 4. Upgrades Applied to Plan

### Must-Have (Release-Blocking)

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | Pager per page in pagination | Task 1, items 5b and 6 | Pager only for single-request path; paginated output uses direct println |

### Strongly Recommended

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | Missing -F -X pager flags | Task 1, item 5a | Changed default from "less -R" to "less -R -F -X" |

### Deferred (Can Safely Defer)

| # | Finding | Rationale for Deferral |
|---|---------|----------------------|
| 1 | ADF nested list indentation | Plan renders listItem content but doesn't specify multi-level nesting depth tracking. Single-level lists work fine. Deep nesting is rare in Atlassian content. |

## 5. Audit & Compliance Readiness

- **Silent failure prevention:** Pager fallback to println ensures output is never lost. ADF non-doc input returns raw JSON string.
- **Deterministic output:** --fields respects user-specified order, giving deterministic column output.
- **Forward compatibility:** Unknown ADF nodes are skipped silently, not errored.

## 6. Final Release Bar

- Pager must not be applied to paginated output (applied).
- Pager default must include -F -X flags (applied).
- Remaining risk: ADF rendering doesn't cover tables, media, or panels. Users see nothing for those nodes. Acceptable because --output json shows the full data.
- I would sign off on this plan.

---

**Summary:** Applied 1 must-have + 1 strongly-recommended upgrade. Deferred 1 item.
**Plan status:** Updated and ready for APPLY

---
*Audit performed by PAUL Enterprise Audit Workflow*
*Audit template version: 1.0*
