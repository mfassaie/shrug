# Jira Issue Bulk Operations

Command reference for bulk operations, issue redaction, comment properties, worklog properties, issue panels, and navigator settings.

All commands follow the pattern:

```
shrug jira "<tag>" <operation> [--param value]
```

---

## issue bulk operations

Bulk issue operations: edit, delete, transition, move, watch/unwatch, and progress tracking.

**10 operations** (3 CRUD + 7 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get bulk editable fields | `--issueIdsOrKeys` (query, required), `--searchText`, `--endingBefore`, `--startingAfter` (all query) |
| `create` | Bulk move issues [body required] | (body only) |
| `get <taskId>` | Get bulk issue operation progress | `--taskId` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `submit-bulk-delete` | POST | Bulk delete issues |
| `submit-bulk-edit` | POST | Bulk edit issues |
| `get-available-transitions` | GET | Get available transitions |
| `submit-bulk-transition` | POST | Bulk transition issue statuses |
| `submit-bulk-unwatch` | POST | Bulk unwatch issues |
| `submit-bulk-watch` | POST | Bulk watch issues |

### Examples

```bash
# Get bulk editable fields for issues
shrug jira "issue bulk operations" list --issueIdsOrKeys PROJ-1,PROJ-2,PROJ-3

# Search editable fields by name
shrug jira "issue bulk operations" list --issueIdsOrKeys PROJ-1,PROJ-2 --searchText "priority"

# Bulk edit issues
echo '{"selectedIssueIdsOrKeys":["PROJ-1","PROJ-2"],"selectedActions":{"priority":{"type":"SET_VALUE","value":"High"}}}' | shrug jira "issue bulk operations" submit-bulk-edit

# Bulk delete issues
echo '{"selectedIssueIdsOrKeys":["PROJ-100","PROJ-101"],"sendBulkNotification":false}' | shrug jira "issue bulk operations" submit-bulk-delete

# Bulk move issues
echo '{"selectedIssueIdsOrKeys":["PROJ-1"],"targetProject":"DEST","targetIssueType":"10001"}' | shrug jira "issue bulk operations" create

# Get available transitions for bulk transition
shrug jira "issue bulk operations" get-available-transitions --issueIdsOrKeys PROJ-1,PROJ-2

# Bulk transition issues
echo '{"selectedIssueIdsOrKeys":["PROJ-1","PROJ-2"],"selectedActions":{"transitionId":"31"}}' | shrug jira "issue bulk operations" submit-bulk-transition

# Bulk watch issues
echo '{"selectedIssueIdsOrKeys":["PROJ-1","PROJ-2","PROJ-3"]}' | shrug jira "issue bulk operations" submit-bulk-watch

# Bulk unwatch issues
echo '{"selectedIssueIdsOrKeys":["PROJ-1","PROJ-2"]}' | shrug jira "issue bulk operations" submit-bulk-unwatch

# Check progress of a bulk operation
shrug jira "issue bulk operations" get task-uuid-here
```

---

## issue redaction

Redact sensitive content from issues and track redaction job status.

**2 operations** (2 CRUD)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `create` | Redact [body required] | (body only) |
| `get <jobId>` | Get redaction status | `--jobId` (path, required) |

### Examples

```bash
# Submit a redaction request
echo '{"issues":[{"issueId":10001,"fieldsToRedact":["description","comment"]}]}' | shrug jira "issue redaction" create

# Check redaction job status
shrug jira "issue redaction" get job-uuid-here
```

---

## issue comment properties

Manage key-value properties on issue comments.

**4 operations** (3 CRUD + 1 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `get <commentId>` | Get comment property keys | `--commentId` (path, required) |
| `update <commentId>` | Set comment property [body required] | `--commentId` (path, required) |
| `delete <commentId>` | Delete comment property | `--commentId` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-comment-property` | GET | Get comment property |

### Examples

```bash
# Get property keys for a comment
shrug jira "issue comment properties" get 10001

# Get a specific comment property
shrug jira "issue comment properties" get-comment-property --commentId 10001 --propertyKey myapp.data

# Set a comment property
echo '{"reviewed":true,"reviewer":"jane"}' | shrug jira "issue comment properties" update 10001 --propertyKey myapp.data

# Delete a comment property
shrug jira "issue comment properties" delete 10001 --propertyKey myapp.data
```

---

## issue worklog properties

Manage key-value properties on issue worklogs.

**4 operations** (3 CRUD + 1 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `get <issueIdOrKey>` | Get worklog property keys | `--issueIdOrKey` (path, required) |
| `update <issueIdOrKey>` | Set worklog property [body required] | `--issueIdOrKey` (path, required) |
| `delete <issueIdOrKey>` | Delete worklog property | `--issueIdOrKey` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-worklog-property` | GET | Get worklog property |

### Examples

```bash
# Get property keys for a worklog
shrug jira "issue worklog properties" get PROJ-123 --worklogId 10001

# Get a specific worklog property
shrug jira "issue worklog properties" get-worklog-property --issueIdOrKey PROJ-123 --worklogId 10001 --propertyKey myapp.billing

# Set a worklog property
echo '{"billable":true,"rate":150}' | shrug jira "issue worklog properties" update PROJ-123 --worklogId 10001 --propertyKey myapp.billing

# Delete a worklog property
shrug jira "issue worklog properties" delete PROJ-123 --worklogId 10001 --propertyKey myapp.billing
```

---

## issue panels

Bulk pin or unpin issue panels to projects (Forge/Connect apps).

**1 operation** (1 CRUD)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `create` | Bulk pin or unpin issue panel to projects [body required] | (body only) |

### Examples

```bash
# Pin an issue panel to projects
echo '{"projectIds":["10000","10001"],"pinned":true}' | shrug jira "issue panels" create
```

---

## issue navigator settings

Manage the default columns shown in the issue navigator.

**2 operations** (1 CRUD + 1 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get issue navigator default columns | (no parameters) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `set-issue-navigator-default-columns` | PUT | Set issue navigator default columns |

### Examples

```bash
# Get the default navigator columns
shrug jira "issue navigator settings" list

# Set the default columns
echo '["issuetype","issuekey","summary","assignee","reporter","priority","status"]' | shrug jira "issue navigator settings" set-issue-navigator-default-columns
```
