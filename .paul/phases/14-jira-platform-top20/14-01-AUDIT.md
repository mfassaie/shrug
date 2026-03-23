# Enterprise Plan Audit Report

**Plan:** .paul/phases/14-jira-platform-top20/14-01-PLAN.md
**Audited:** 2026-03-23
**Verdict:** Conditionally acceptable (now enterprise-ready after applied upgrades)

---

## 1. Executive Verdict

The plan is well-scoped E2E test expansion following an established, proven pattern. The original plan had one release-blocking gap (AC-1 claimed content verification but the task only checked HTTP success) and several specificity gaps that could produce flaky or misleading test results in production environments.

After applying 5 upgrades, the plan is enterprise-ready. I would approve this for production.

## 2. What Is Solid

- **Scope discipline.** Three entities, three tasks, one file. No scope creep into source code or harness changes. Boundaries are explicit and correct.
- **Pattern reuse.** Every task follows the established jira.rs convention (skip_unless_e2e, setup_profile, rate_limit_delay, cleanup). Reduces maintenance burden and review friction.
- **Pre-planned failure handling for votes.** The vote test acknowledges the Jira "own issue" restriction upfront rather than discovering it at runtime. This prevents false negatives.
- **Issue link test creates two issues.** Correctly avoids self-linking which some Jira instances reject.
- **Append-only boundary.** Existing tests protected from accidental modification.

## 3. Enterprise Gaps Identified

1. **AC-1 vs Task 1 mismatch (content verification).** AC-1 states the watcher list should "contain that accountId" but the task only asserted the HTTP call succeeded. In a live environment with eventual consistency, a 200 response with an empty watchers list would pass the original task but violate AC-1.

2. **accountId extraction without guard.** Task 1 extracted accountId from `fields.reporter.accountId` but had no fallback if the field was null or missing. API authentication issues or permission changes could produce an issue JSON without a reporter block, causing an unhelpful panic.

3. **Vote error code unspecified.** The original plan said "check exit_code and stderr for 404 or permission error" but didn't specify the actual HTTP status Jira returns. Vague error matching risks either swallowing real failures or failing on expected errors.

4. **Link ID extraction via blind [0] index.** Using `issuelinks[0].id` assumes the newly created link is the first in the array. If the test project has any pre-existing links (from failed prior runs, manual testing, or other automation), index [0] could return a stale link. This would make get-issue-link succeed on wrong data and delete-issue-link delete something it shouldn't.

5. **Empty link types not handled.** If issue linking is disabled on the Jira instance (an admin setting), `get-issue-link-types` returns an empty list. The test would panic on array access rather than skipping gracefully.

6. **Test cleanup not panic-safe.** All three tests place cleanup (delete_issue, teardown_profile) at the end of the function. A mid-test assertion failure panics and skips cleanup, orphaning resources in Jira. This matches the existing pattern across all jira.rs tests, and fixing it would require harness changes (out of scope), so this is noted but deferred.

## 4. Upgrades Applied to Plan

### Must-Have (Release-Blocking)

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 1 | AC-1 requires content verification but Task 1 only checked HTTP success | Task 1 action step 4 | Added requirement to parse watcher list JSON and verify it contains the expected accountId |

### Strongly Recommended

| # | Finding | Plan Section Modified | Change Applied |
|---|---------|----------------------|----------------|
| 2 | accountId extraction had no null guard | Task 1 action step 2 | Added assertion that accountId is present and non-empty, with early return if missing |
| 3 | Vote error codes were vague | Task 2 action (error handling note) | Specified HTTP 404 as the expected Jira response, with stderr matching guidance |
| 4 | Link ID extraction via blind [0] was fragile | Task 3 action step 4 | Changed to filter issuelinks array by matching the other test issue's key |
| 5 | Empty link types would cause panic | Task 3 action step 2 | Added graceful early return if no link types available |

### Deferred (Can Safely Defer)

| # | Finding | Rationale for Deferral |
|---|---------|----------------------|
| 1 | Test cleanup not panic-safe (orphaned resources on assertion failure) | Matches the established pattern across all existing jira.rs tests. Fixing requires harness-level changes (adding Drop guards or panic hooks), which is out of scope for this plan and protected by boundaries. Could be addressed in a dedicated test infrastructure phase. |

## 5. Audit & Compliance Readiness

**Evidence production:** Each test produces pass/fail with eprintln trace of created/deleted resource IDs. Sufficient for post-incident reconstruction of test runs.

**Silent failure prevention:** The key upgrade (Finding #1) closes the only path where a test could pass while the feature was broken. The watcher content verification ensures a 200 with empty data is caught.

**Ownership:** Single file modification, clear pattern, no cross-cutting concerns. Any Rust developer can review and maintain these tests.

**Known limitation:** Orphaned resources on test failure (deferred item #1) could accumulate in the live Jira instance over time. For a test account this is acceptable. For a shared production-adjacent environment, a periodic cleanup script would be needed.

## 6. Final Release Bar

**What must be true before this plan ships:**
- All 5 applied upgrades are implemented in the test code (not just in the plan)
- Tests compile with zero clippy warnings
- Watcher test verifies response content, not just HTTP status

**Risks if shipped as original plan:**
- Watcher test could pass while add-watcher silently fails (200 with empty list)
- Link test could delete wrong link in a project with existing links
- Both would produce misleading "all tests pass" results

**Sign-off:** With the applied upgrades, I would sign my name to this plan. The scope is appropriate, the patterns are proven, and the gaps have been closed.

---

**Summary:** Applied 1 must-have + 4 strongly-recommended upgrades. Deferred 1 item.
**Plan status:** Updated and ready for APPLY

---
*Audit performed by PAUL Enterprise Audit Workflow*
*Audit template version: 1.0*
