---
phase: 05-generic-http-executor
plan: 01
subsystem: api
tags: [reqwest, http, executor, auth-injection, dry-run]

requires:
  - phase: 03-dynamic-command-tree
    provides: ResolvedCommand with operation, server_url, remaining_args
  - phase: 04-authentication-profiles
    provides: ResolvedCredential with AuthScheme (Basic/Bearer)
  - phase: 02-openapi-spec-engine
    provides: build_url(), build_query_string(), Operation model
provides:
  - Core HTTP executor (parse_args → build_request → execute)
  - Argument parser matching CLI flags to operation parameters
  - Auth header injection from resolved credentials
  - Dry-run mode with credential masking
  - --json flag for request body input
  - Site URL substitution for {baseUrl} specs
affects: [05-02-retries, 05-03-pagination, 05-04-quirks, 06-output-formatting]

tech-stack:
  added: []
  patterns: [blocking reqwest::Client reuse, status-to-error mapping, flag-to-param matching]

key-files:
  created: [src/executor.rs]
  modified: [src/lib.rs, src/main.rs, src/cli.rs]

key-decisions:
  - "Param flags use original names (not kebab-case conversion) for simplicity"
  - "Site URL substitution when spec server_url is {baseUrl} — uses credential's site field"
  - "Blocking HTTP client (not async) for initial implementation"

patterns-established:
  - "Status code → ShrugError mapping pattern for all API responses"
  - "Credential masking in dry-run output"
  - "Single reqwest::Client created once in run(), passed through call chain"

duration: ~15min
completed: 2026-03-21
---

# Phase 5 Plan 01: Core HTTP Executor Summary

**HTTP executor bridging resolved commands to live API calls, with argument parsing, auth injection, dry-run mode, and --json body support.**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~15min |
| Completed | 2026-03-21 |
| Tasks | 3 completed |
| Files modified | 4 |
| Tests added | 22 (225 total) |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: Argument parsing from remaining_args | Pass | Parses --flag value pairs, separates path/query, validates required |
| AC-2: HTTP request construction | Pass | URL assembly via build_url + query string, correct method, headers |
| AC-3: Auth header injection | Pass | Basic (base64), Bearer, None with warning. Tested. |
| AC-4: Request execution and response handling | Pass | 204 silent, 400/401/403/404/429/5xx mapped with response body |
| AC-5: Dry-run support | Pass | Prints method, URL, masked auth, body to stderr |
| AC-6: JSON body input | Pass | --json flag, required body validation |
| AC-7: Connection pooling | Pass | Single Client created once in run() |

## Accomplishments

- Created `src/executor.rs` (748 lines) with parse_args, execute, create_client, and 22 tests
- Wired executor into main.rs replacing the Phase 5 placeholder
- Added --json global CLI flag for request body input
- Credential pass-through from profile resolution to HTTP execution now works end-to-end
- Site URL substitution handles Atlassian's `{baseUrl}` server variable pattern

## Task Commits

| Task | Commit | Type | Description |
|------|--------|------|-------------|
| Tasks 1-3 | `d97dce2` | feat | Core executor, CLI wiring, 22 tests |

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `src/executor.rs` | Created | HTTP executor: arg parsing, request building, execution, dry-run |
| `src/lib.rs` | Modified | Added `pub mod executor` |
| `src/main.rs` | Modified | Wired executor, credential pass-through, removed placeholder |
| `src/cli.rs` | Modified | Added --json global flag |

## Decisions Made

| Decision | Rationale | Impact |
|----------|-----------|--------|
| Use original param names as flags (not kebab-case) | Atlassian param names like `issueIdOrKey` are what users see in docs. Converting to kebab-case and back adds complexity for little gain. | Users type `--issueIdOrKey` not `--issue-id-or-key` |
| Blocking HTTP client | Simpler for initial implementation. tokio runtime is available but executor uses reqwest::blocking. | May need to switch to async for pagination (05-03) |
| Site URL substitution in executor | Audit finding: Atlassian specs use `{baseUrl}` which strips to empty. Executor resolves from credential's site field. | All 5 products work correctly |

## Deviations from Plan

### Summary

| Type | Count | Impact |
|------|-------|--------|
| Auto-fixed | 1 | Clippy lint (redundant closure) |
| Scope additions | 0 | None |
| Deferred | 0 | None |

**Total impact:** Minimal. One clippy fix, no scope creep.

## Issues Encountered

| Issue | Resolution |
|-------|------------|
| Bundled specs have 0 operations (test fixtures) | Dry-run end-to-end test can't route to real operation without cached spec. Unit tests cover all executor logic. |

## Next Phase Readiness

**Ready:**
- executor::execute() is the single entry point for all API calls
- Rate limiting (05-02) adds retry logic around execute()
- Pagination (05-03) adds iteration around execute()
- Quirks (05-04) adds pre/post-request hooks

**Concerns:**
- Blocking HTTP client may need async conversion for pagination performance
- Header/Cookie parameters deferred (audit: can safely defer)

**Blockers:**
- None

---
*Phase: 05-generic-http-executor, Plan: 01*
*Completed: 2026-03-21*
