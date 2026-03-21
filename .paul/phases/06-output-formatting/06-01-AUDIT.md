# Enterprise Plan Audit Report

**Plan:** .paul/phases/06-output-formatting/06-01-PLAN.md
**Audited:** 2026-03-21
**Verdict:** Enterprise-ready after fixes applied

---

## 1. Executive Verdict

Conditionally acceptable, now enterprise-ready after applying 2 findings. The plan is well-structured with clear formatter boundaries and sensible TTY detection logic. The two gaps were both about input resilience: what happens when the real world delivers non-JSON responses, and whether CSV output is deterministic. Both applied. I would approve this for production.

## 2. What Is Solid

- **format_response returning String.** Decouples formatting from I/O. The executor controls when and where to write, not the formatter. This is correct for testing and future pager integration.
- **TTY auto-detection with explicit override.** resolve_format() handles the auto case cleanly. Users who pipe output get JSON. Users in a terminal get tables. Explicit --output overrides both. This is the standard pattern (cf. `gh`, `kubectl`).
- **NO_COLOR spec compliance.** Checking the environment variable in should_use_color() follows the no-color.org specification.
- **Scope limits are appropriate.** Deferring ADF, pager, and --fields to 06-02 keeps this plan focused on the five core formatters.
- **Known results array extraction.** Reusing the same "issues/values/results" pattern from pagination means table and CSV output will correctly find the data array in Atlassian wrapper objects.

## 3. Enterprise Gaps Identified

### Gap 1: Non-JSON response body handling (crash risk)
format_response() specifies "Parses body as serde_json::Value" with no fallback. In production, the executor can receive:
- HTML error pages (Atlassian returns HTML for some 5xx errors)
- Empty bodies (204 already handled, but edge cases exist)
- Plain text error messages
If serde_json::from_str fails and the function panics or returns an empty string, users lose diagnostic information.

### Gap 2: Non-deterministic CSV column ordering
CSV with headers from "first object's keys" produces output whose column order depends on JSON parsing order. While serde_json preserves insertion order, this is an implementation detail. Different API responses for the same resource may order keys differently. For scripting and diff-friendliness, columns should be sorted.

## 4. Upgrades Applied to Plan

### Must-Have (Release-Blocking)

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | Non-JSON body crashes format_response | Task 1, item 2a (format_response spec) | Added: if JSON parsing fails, return raw body unchanged |

### Strongly Recommended

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | Non-deterministic CSV columns | Task 1, item 2e (format_csv spec) | Added: headers sorted alphabetically for deterministic output |

### Deferred (Can Safely Defer)

| # | Finding | Rationale for Deferral |
|---|---------|----------------------|
| 1 | Table column width could use terminal width | Plan explicitly defers this. Simple truncation at 60 chars is acceptable for v1. Terminal-aware sizing is a UX polish item. |

## 5. Audit & Compliance Readiness

- **Silent failure prevention:** The JSON parse fallback ensures no data is silently lost. Users always see the response, even if it cannot be formatted.
- **Deterministic output:** Sorted CSV columns produce reproducible output suitable for automated comparison in CI/CD pipelines.
- **Audit trail:** The --output flag and TTY detection are logged at debug level by the executor's existing tracing infrastructure.

## 6. Final Release Bar

- The JSON parse fallback must be in place (applied). Without it, a single HTML error page from Atlassian could panic the formatter.
- CSV column sorting must be consistent (applied). Non-deterministic output breaks downstream scripts.
- Remaining risk: table truncation at 60 chars may cut important data. Acceptable for v1 since users can switch to JSON for full output.
- I would sign off on this plan.

---

**Summary:** Applied 1 must-have + 1 strongly-recommended upgrade. Deferred 1 item.
**Plan status:** Updated and ready for APPLY

---
*Audit performed by PAUL Enterprise Audit Workflow*
*Audit template version: 1.0*
