# Enterprise Plan Audit Report

**Plan:** .paul/phases/02-openapi-spec-engine/02-02-PLAN.md
**Audited:** 2026-03-21
**Verdict:** Enterprise-ready after fixes applied

---

## 1. Executive Verdict

**Conditionally acceptable → enterprise-ready after applying 2 must-have + 2 strongly-recommended fixes.**

The plan demonstrates solid architectural thinking: converting Swagger 2.0 directly to the existing ApiSpec model (not a full OpenAPI 3.0 intermediate), consistent patterns with the v3 parser, and clear test coverage expectations. However, the original plan had gaps in how Swagger 2.0's `consumes` inheritance and multi-param body handling work, which would produce incorrect RequestBody content_types in production use against real BitBucket specs.

I would approve this plan for production after the applied fixes.

## 2. What Is Solid (Do Not Change)

- **Direct-to-ApiSpec conversion strategy.** Not doing a full Swagger 2.0 → OpenAPI 3.0 intermediate step keeps the parser simple and avoids importing a large conversion library. The ApiSpec model is purpose-built for CLI generation, making this the right architectural call.

- **Parameter merge reuse from v3.** The plan correctly identifies that merge_parameters is the same semantics in both spec versions and reuses the pattern rather than reimplementing.

- **Skip-without-operationId behavior.** Consistent with v3 parser — warn and skip rather than fail. This is correct for real-world specs that may have undocumented operations.

- **Boundaries are correctly scoped.** No model changes, no $ref resolution, no network fetching. The parser stays in its lane.

- **Test mirroring approach.** Structuring swagger tests to parallel v3 tests ensures consistent coverage expectations.

## 3. Enterprise Gaps Identified

### Gap 1: Multiple formData params → RequestBody ambiguity (CRITICAL)
Swagger 2.0 operations frequently have multiple `"in": "formData"` parameters (e.g., BitBucket file upload: `file` + `message` + `branch`). The original plan implied each formData param gets its own RequestBody, but `ApiSpec::Operation::request_body` is `Option<RequestBody>` — only one. Without explicit merge semantics, only the last formData param would survive, silently dropping others.

### Gap 2: Spec-level `consumes` inheritance (CRITICAL)
Swagger 2.0 has a root-level `"consumes"` array that all operations inherit unless they override with their own `"consumes"`. The original plan only checked operation-level `"consumes"`, meaning most operations in a real spec (which don't override) would get hardcoded `["application/json"]` instead of the spec's declared content types.

### Gap 3: `schemes` array ignored
The original plan hardcoded `"https://"` for server_url construction. Swagger 2.0 has a `"schemes"` array (`["https", "http"]`). While BitBucket does use HTTPS, hardcoding violates the principle of parsing what the spec says. A non-Atlassian Swagger 2.0 spec fed through this parser would get incorrect URLs.

### Gap 4: basePath normalization
Combining `host` + `basePath` without normalization risks double-slashes (e.g., `api.example.com/` + `/2.0/` → `https://api.example.com//2.0/`). Edge case, but it produces incorrect URLs silently.

## 4. Upgrades Applied to Plan

### Must-Have (Release-Blocking)

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | Multiple formData params must merge into single RequestBody; body/formData params must be excluded from parameters vec | AC-2a added, Task 1 action rewritten for body/formData handling | Added AC-2a for multi-formData → single RequestBody. Task 1 now specifies: all formData params in one operation produce ONE RequestBody, required=true if any param is required, formData/body params excluded from parameters vec. Added test case. |
| 2 | Spec-level `consumes` inheritance for content_types | AC-2b added, Task 1 action updated | Added AC-2b requiring spec-level consumes to be read once and inherited by operations lacking their own. Task 1 now reads spec-level consumes at parser top, uses it as default for body/formData content_types. Added test case. |

### Strongly Recommended

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | `schemes` array for server_url scheme selection | AC-2 updated, Task 1 step 3 rewritten | AC-2 now specifies `{scheme}://` from schemes array (prefer https, fall back to first, default https). Task 1 step 3 updated with scheme resolution logic. Added test case. |
| 2 | basePath normalization to prevent double-slash | Task 1 step 3 updated | Added normalization: strip trailing slash from host, ensure basePath starts with `/`. Added test case. |

### Deferred (Can Safely Defer)

| # | Finding | Rationale for Deferral |
|---|---------|----------------------|
| 1 | Double JSON parse in `parse_spec()` (parses to Value for detection, then each parser parses string again) | Plan 02-03 introduces rkyv caching, making JSON parse performance irrelevant for warm starts. Cold starts with bundled specs parse once. If profiling in 02-04 shows this matters, a `parse_from_value()` variant can be added then. Not worth changing v3 parser's public API now. |
| 2 | `produces` field extraction | ApiSpec does not model response content types. No downstream consumer needs this. Can be added if output formatting (Phase 6) requires content-type awareness. |

## 5. Audit & Compliance Readiness

**Evidence production:** The plan produces testable artifacts — unit tests with realistic fixtures prove correctness. The test list now covers the critical edge cases (multi-formData, consumes inheritance, schemes, normalization).

**Silent failure prevention:** The consumes inheritance fix prevents silent wrong-content-type bugs that would only manifest as 415 errors in production against real APIs. The formData merge fix prevents silent parameter loss.

**Post-incident reconstruction:** Parser is deterministic (JSON in → ApiSpec out). Any production bug can be reproduced by saving the input spec JSON and re-running the parser. No external state dependencies.

**Ownership:** Single module (`src/spec/swagger.rs`) with clear scope boundaries. Maintainable by anyone who understands the v3 parser.

## 6. Final Release Bar

**What must be true before this ships:**
- All 5 original ACs + 2 audit-added ACs pass
- Multi-formData test proves N formData params → 1 RequestBody
- Consumes inheritance test proves spec-level → operation-level fallback
- No regressions in existing v3 parser tests

**Remaining risks if shipped as-is (after fixes):**
- Double JSON parse is a minor performance concern for very large specs on cold start. Mitigated by caching in 02-03.
- No real BitBucket spec validation yet (deferred to 02-04 conformance tests). The fixture-based tests are synthetic.

**Sign-off:** With the applied fixes, I would sign my name to this plan. The Swagger 2.0 spec's key structural differences (consumes inheritance, body/formData semantics, schemes) are now correctly handled.

---

**Summary:** Applied 2 must-have + 2 strongly-recommended upgrades. Deferred 2 items.
**Plan status:** Updated and ready for APPLY

---
*Audit performed by PAUL Enterprise Audit Workflow*
*Audit template version: 1.0*
