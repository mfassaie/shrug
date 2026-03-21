# Enterprise Plan Audit Report

**Plan:** .paul/phases/05-generic-http-executor/05-04-PLAN.md
**Audited:** 2026-03-21
**Verdict:** Enterprise-ready after fixes applied

---

## 1. Executive Verdict

Conditionally acceptable, now enterprise-ready after applying 2 findings. The plan is well-scoped, appropriately bounded, and targets a real operational gap (silent 403s on attachment endpoints). The two issues found were a compile-blocking type error and a missing verification step. Both have been applied. I would approve this for production.

## 2. What Is Solid

- **Static match lookup over HashMap/lazy_static.** Zero allocation, zero initialization cost, exhaustive by construction. Correct choice for a small, compile-time-known registry.
- **Product + operationId compound key.** Prevents false matches when different products share operationId naming patterns. The match structure makes this impossible to get wrong.
- **Boundaries are correct.** No response-side quirks, no multipart handling, no runtime loading. The plan knows what it is not doing and says so explicitly.
- **Debug logging on quirk application.** Gives operators visibility into when quirks fire without polluting normal output.
- **Scope limits prevent creep.** "Do not add quirks for operations we cannot verify from documentation" is a good policy that prevents speculative entries.

## 3. Enterprise Gaps Identified

### Gap 1: Static type mismatch (compile-blocking)
The `Quirk` struct specified `extra_headers: Vec<(&'static str, &'static str)>`. Since `get_quirk` returns `Option<&'static Quirk>` and the quirks are defined as `static` constants, `Vec` cannot be used (it requires heap allocation and is not const-constructible). This would fail to compile.

### Gap 2: Unverified operationId assumptions
The plan lists specific operationIds (`addAttachment`, `createAttachment`, `updateAttachment`) but provides no verification that these IDs actually exist in the bundled specs. If any ID is wrong (typo, renamed, different casing), the quirk entry is dead code that silently never fires. This creates a false sense of coverage.

## 4. Upgrades Applied to Plan

### Must-Have (Release-Blocking)

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | Vec cannot be used in static Quirk | Task 1, item 1 (Quirk struct definition) | Changed `Vec<(&'static str, &'static str)>` to `&'static [(&'static str, &'static str)]` |

### Strongly Recommended

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | OperationIds not verified against specs | Task 1, item 5 (unit tests) | Added test: registered operationIds must exist in bundled specs |

### Deferred (Can Safely Defer)

| # | Finding | Rationale for Deferral |
|---|---------|----------------------|
| 1 | Header precedence documentation | reqwest's `.header()` replaces same-name headers, so quirk headers naturally override defaults. Behaviour is correct by construction. Adding a code comment would be nice but not blocking. |

## 5. Audit & Compliance Readiness

- **Audit evidence:** The operationId verification test (strongly-recommended finding) ensures the registry is truthful. Without it, a compliance reviewer could not distinguish between "quirk exists" and "quirk works".
- **Silent failure prevention:** The debug log on quirk application means operators can verify quirks are firing in production via trace output.
- **Post-incident reconstruction:** If a 403 occurs on an attachment endpoint, debug logs will show whether the quirk was applied. If absent from logs, the registry lookup is the first investigation target.
- **Ownership:** The match-based registry is a single file with clear, greppable entries. No indirection.

## 6. Final Release Bar

- The static slice type fix must be in place (applied). Without it, the code does not compile.
- The operationId verification test must pass against current bundled specs (applied as test requirement).
- Remaining risk: if Atlassian renames operationIds in a future spec update, quirks silently stop firing. The verification test would catch this during builds if bundled specs are also updated. Acceptable.
- I would sign off on this plan.

---

**Summary:** Applied 1 must-have + 1 strongly-recommended upgrade. Deferred 1 item.
**Plan status:** Updated and ready for APPLY

---
*Audit performed by PAUL Enterprise Audit Workflow*
*Audit template version: 1.0*
