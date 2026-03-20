# Research: Atlassian OpenAPI Specs

**Date:** 2026-03-21
**Source:** Atlassian Cloud REST API specifications
**Agent:** general-purpose (web)

---

## Summary

All five Atlassian products have publicly available API specs. Four use OpenAPI 3.0.1; Bitbucket is the outlier on Swagger 2.0. Combined surface area: ~1,250 operations, ~1,523 schemas.

## Spec Inventory

| Product | Spec URL | OpenAPI | Paths | Ops | Schemas | Tags |
|---------|----------|---------|-------|-----|---------|------|
| Jira Platform | `dac-static.atlassian.com/cloud/jira/platform/swagger-v3.v3.json` | 3.0.1 | 420 | 620 | 972 | 100 |
| Jira Software | `dac-static.atlassian.com/cloud/jira/software/swagger.v3.json` | 3.0.1 | 68 | 95 | 64 | 13 |
| Jira Service Mgmt | `dac-static.atlassian.com/cloud/jira/service-desk/swagger.v3.json` | 3.0.1 | 45 | 70 | 110 | 8 |
| Confluence | `dac-static.atlassian.com/cloud/confluence/swagger.v3.json` | 3.0.1 | 89 | 130 | 168 | 28 |
| Bitbucket | `bitbucket.org/api/swagger.json` | **2.0** | 193 | 335 | 209 | 23 |

## Jira Platform Deep Dive (largest spec)

### Structure
- OpenAPI 3.0.1, 2.47 MB
- Server: `https://your-domain.atlassian.net`
- All paths under `/rest/api/3/`
- 620 operations (GET: 275, POST: 136, PUT: 119, DELETE: 90)
- 972 schemas, 100 tags
- 38 deprecated ops (6.1%), 119 experimental ops (19.2%)

### Authentication
Two schemes, both accepted by most operations:
1. **OAuth2** (authorization code) — `auth.atlassian.com/authorize` + `/oauth/token`
   - ~170 granular scopes: `read:issue:jira`, `write:project:jira`, etc.
   - Legacy coarse scopes: `read:jira-work`, `write:jira-work`
2. **Basic Auth** — email + API token

### Pagination (two styles)
- **Offset-based** (dominant, 83 ops): `startAt` + `maxResults` → `PageBean*` response with `values[]`, `total`, `isLast`
- **Cursor-based** (8 ops): `nextPageToken`
- 61 distinct PageBean schemas (PageBeanProject, PageBeanChangelog, etc.)

### Common Parameters
| Param | Frequency | Notes |
|-------|-----------|-------|
| maxResults | 95 ops | Pagination |
| startAt | 83 ops | Pagination |
| expand | 68 ops | Include nested data |
| projectId | 25 ops | Scope to project |
| accountId | 23 ops | Scope to user |
| query | 22 ops | Free text search |
| orderBy | 19 ops | Sort results |

### Resource Hierarchy
Top resources by path count:
- `/rest/api/3/issue` (35 paths) — Issues, transitions, watchers, comments
- `/rest/api/3/project` (35 paths) — Projects, roles, components
- `/rest/api/3/field` (29 paths) — Fields, contexts, options
- `/rest/api/3/user` (17 paths) — Users, search, properties

### Error Format
Consistent `ErrorCollection` schema:
```json
{"errorMessages": ["string"], "errors": {"fieldName": "message"}, "status": 422}
```

### operationId
Every operation has a unique `operationId` (e.g., `createIssue`, `getIssue`, `searchForIssuesUsingJql`) — ideal for CLI command naming.

## Challenges for Dynamic CLI Generation

### Manageable
- Clean RESTful structure, consistent patterns
- operationIds map to CLI commands
- Tags map to command groups
- Well-typed parameters with descriptions
- Consistent auth and error handling

### Requires Design Decisions
1. **Dynamic issue fields** — `IssueBean.fields` uses `additionalProperties: {}` (custom fields are not statically typed)
2. **Two pagination styles** need unified CLI handling
3. **`expand` parameter** — valid values are in descriptions, not enums
4. **Deprecated/experimental ops** — need filtering or flagging
5. **Spec size** (2.47 MB for Jira alone) — caching/pre-processing needed
6. **Bitbucket outlier** — Swagger 2.0 needs conversion or separate parser
7. **972 schemas, 61 PageBean variants** — need smart simplification for CLI UX

### Structural Differences from Google Discovery

| Aspect | Google Discovery | Atlassian OpenAPI |
|--------|-----------------|-------------------|
| Format | Proprietary REST Description | OpenAPI 3.0.1 |
| Resources | Nested in `resources{}` | Flat path strings |
| Parameters | In `methods[].parameters` | In `parameters[]` array |
| Refs | `"$ref": "Name"` | `$ref: '#/components/schemas/Name'` |
| Tags | Implicit from resource nesting | Explicit `tags[]` on operations |
| CLI mapping | Resource tree → subcommands | Tags → command groups, operationId → commands |

## Cross-Product Observations

- Jira Platform is 50% of total API surface
- Jira Software and JSM extend Jira (same auth, same domain)
- Confluence follows identical patterns (3.0.1, same auth)
- Bitbucket is architecturally separate (different domain, Swagger 2.0, different auth options)
- URL pattern: `https://dac-static.atlassian.com/cloud/{product}/swagger.v3.json`

---
*Research completed: 2026-03-21*
