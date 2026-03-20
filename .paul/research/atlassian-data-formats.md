# Research: Atlassian Data Formats & Patterns

**Date:** 2026-03-21
**Agent:** general-purpose (web)

---

## Summary

Atlassian uses ADF (Atlassian Document Format) for rich text, JQL for search, two pagination styles, and a points-based rate limiting model. Key design decisions: accept Markdown input → convert to ADF, abstract pagination into a unified iterator, auto-retry on 429.

## 1. Atlassian Document Format (ADF)

### What It Is
JSON-based rich text format used in Jira descriptions, comments, and Confluence pages.

### Structure
```json
{
  "version": 1,
  "type": "doc",
  "content": [
    {
      "type": "paragraph",
      "content": [
        { "type": "text", "text": "Hello " },
        { "type": "text", "text": "world", "marks": [{ "type": "strong" }] }
      ]
    }
  ]
}
```

### Node Types
- **Block:** paragraph, heading, bulletList, orderedList, codeBlock, blockquote, table, panel, rule, expand
- **Inline:** text, mention, emoji, date, status, hardBreak, inlineCard
- **Marks:** strong, em, underline, strike, code, link, textColor

### CLI Strategy
- **Input:** Accept Markdown or plain text → convert to ADF. Simplest ADF = text wrapped in a paragraph node.
- **Output:** Walk ADF tree, render as plain text or ANSI-formatted terminal output.
- **Libraries (JS):** `md-to-adf`, `marklassian`, `extended-markdown-adf-parser`
- For Rust: likely need a custom Markdown → ADF converter (or use a simple text → paragraph wrapper)

## 2. Wiki Markup (Legacy)

- Confluence Cloud v2 API uses ADF. Storage format (XHTML) works with v1 API but is deprecated.
- Wiki markup conversion endpoints deprecated October 2024.
- **Decision: Target ADF only.** No wiki markup support.

## 3. JQL (Jira Query Language)

### Syntax
`field operator value` with AND/OR/NOT, ORDER BY

### Operators
`=`, `!=`, `>`, `>=`, `<`, `<=`, `~` (contains), `!~`, `IN`, `NOT IN`, `IS`, `IS NOT`, `WAS`, `CHANGED`

### Functions
`currentUser()`, `startOfDay()`, `endOfWeek()`, `membersOf("group")`, `updatedBy(user, daterange)`

### CLI Approach
- `--jql` flag for raw JQL: `shrug jira search --jql 'project = TEST AND status != Done'`
- Shorthand flags that build JQL internally: `--project KEY`, `--assignee me`, `--status "In Progress"`
- JQL autocomplete/validation API available at `/rest/api/3/jql/match`

## 4. Data Patterns

### Issue Fields
- System: summary, description (ADF), status, assignee, reporter, priority, labels, components
- Custom: `customfield_NNNNN` — field metadata API maps human names to IDs
- **CLI:** Cache field metadata, allow both human names and customfield_ IDs

### User Identification
- `accountId` is canonical (GDPR). Email/displayName available but read-only for user fields.
- `shrug user search "display name"` → returns accountIds

### Workflow Transitions
- GET transitions → POST transition by ID
- **CLI:** `shrug issue transition KEY --to "In Progress"` resolves transition ID automatically

### Attachments
- Multipart POST to `/rest/api/3/issue/{key}/attachments`
- Requires `X-Atlassian-Token: no-check` header

## 5. Date/Time Formats

- Date only: `YYYY-MM-DD`
- Date-time: ISO 8601 `YYYY-MM-DDThh:mm:ss.sTZD`
- JQL relatives: `-7d`, `startOfDay()`, `endOfWeek("+1")`
- **CLI:** Store ISO 8601, display relative ("3 hours ago"), accept relative input (`--since 7d`)

## 6. Pagination (Unified Strategy)

| Product | Page Param | Size Param | Total? | Next Mechanism |
|---------|-----------|------------|--------|---------------|
| Jira (legacy) | startAt | maxResults | Yes | Calculate offset |
| Jira (new) | nextPageToken | maxResults | No | Token |
| Confluence v2 | cursor | limit | No | Link header |
| Bitbucket | page | pagelen | Yes | next URL |

**CLI:** Abstract behind unified iterator. Default to fetching all with `--limit` cap. Progress indicator when total known, spinner when not. `--page-size N` for per-request control.

## 7. Rate Limiting

### Three Independent Systems (Jira)
1. **Points-based quota:** 65,000-500,000 pts/hr (but API token traffic exempt!)
2. **Burst limits:** GET/POST 100 RPS, PUT/DELETE 50 RPS
3. **Per-issue write:** 20 ops/2s, 100 ops/30s

### Response Headers
- `Retry-After` — seconds to wait on 429
- `X-RateLimit-Remaining` — remaining capacity
- `X-RateLimit-NearLimit` — true when <20% left
- `RateLimit-Reason` — which limit was hit

### CLI Backoff Strategy
```
On 429: read Retry-After; if absent: exponential backoff 2s → 4s → 8s → 16s
Jitter: delay × random(0.7, 1.3)
Max retries: 4
Proactive: slow batch ops when X-RateLimit-NearLimit is true
```

**Key:** API token auth = burst limits only (not points-based). Simplifies rate limit handling for shrug.

---
*Research completed: 2026-03-21*
