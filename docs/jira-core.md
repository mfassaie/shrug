# Jira Core Entities

Command reference for the core Jira entities: issues, projects, dashboards, filters, labels, issue search, and JQL.

All commands follow the pattern:

```
shrug jira "<tag>" <operation> [--param value]
```

CRUD operations (list, create, get, update, delete) are mapped automatically. Raw operations use their full operation name.

---

## issues

Manage Jira issues: create, read, update, delete, plus transitions, changelogs, archiving, and bulk operations.

**22 operations** (5 CRUD + 17 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get issue limit report | `--isReturningKeys` (query) |
| `create` | Create issue [body required] | `--updateHistory` (query) |
| `get <issueIdOrKey>` | Get issue | `--issueIdOrKey` (path, required), `--fields`, `--fieldsByKeys`, `--expand`, `--properties`, `--updateHistory`, `--failFast` (all query) |
| `update <issueIdOrKey>` | Edit issue [body required] | `--issueIdOrKey` (path, required), `--notifyUsers`, `--overrideScreenSecurity`, `--overrideEditableFlag`, `--returnIssue`, `--expand` (all query) |
| `delete <issueIdOrKey>` | Delete issue | `--issueIdOrKey` (path, required), `--deleteSubtasks` (query) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-bulk-changelogs` | POST | Bulk fetch changelogs |
| `get-events` | GET | Get events |
| `archive-issues-async` | POST | Archive issue(s) by JQL |
| `archive-issues` | PUT | Archive issue(s) by issue ID/key |
| `create-issues` | POST | Bulk create issue |
| `bulk-fetch-issues` | POST | Bulk fetch issues |
| `get-create-issue-meta-issue-types` | GET | Get create metadata issue types for a project |
| `get-create-issue-meta-issue-type-id` | GET | Get create field metadata for a project and issue type id |
| `unarchive-issues` | PUT | Unarchive issue(s) by issue keys/ID |
| `assign-issue` | PUT | Assign issue |
| `get-change-logs` | GET | Get changelogs |
| `get-change-logs-by-ids` | POST | Get changelogs by IDs |
| `get-edit-issue-meta` | GET | Get edit issue metadata |
| `notify` | POST | Send notification for issue |
| `get-transitions` | GET | Get transitions |
| `do-transition` | POST | Transition issue |
| `export-archived-issues` | PUT | Export archived issue(s) |

#### get-transitions parameters

| Parameter | Location | Required |
|-----------|----------|----------|
| `--issueIdOrKey` | path | yes |
| `--expand` | query | no |
| `--transitionId` | query | no |
| `--skipRemoteOnlyCondition` | query | no |
| `--includeUnavailableTransitions` | query | no |
| `--sortByOpsBarAndStatus` | query | no |

#### do-transition parameters

| Parameter | Location | Required |
|-----------|----------|----------|
| `--issueIdOrKey` | path | yes |

### Examples

```bash
# Get an issue by key
shrug jira issues get PROJ-123

# Get an issue with specific fields
shrug jira issues get PROJ-123 --fields summary,status,assignee

# Create an issue (pipe JSON body via stdin)
echo '{"fields":{"project":{"key":"PROJ"},"summary":"Bug title","issuetype":{"name":"Bug"}}}' | shrug jira issues create

# Update an issue without notifying watchers
shrug jira issues update PROJ-123 --notifyUsers false < body.json

# Delete an issue and its subtasks
shrug jira issues delete PROJ-123 --deleteSubtasks true

# Get available transitions for an issue
shrug jira issues get-transitions --issueIdOrKey PROJ-123

# Transition an issue to a new status
echo '{"transition":{"id":"31"}}' | shrug jira issues do-transition --issueIdOrKey PROJ-123

# Assign an issue
echo '{"accountId":"5b10ac8d82e05b22cc7d4ef5"}' | shrug jira issues assign-issue --issueIdOrKey PROJ-123

# Get changelogs for an issue
shrug jira issues get-change-logs --issueIdOrKey PROJ-123
```

---

## projects

Manage Jira projects: CRUD, archiving, status listing, and hierarchy queries.

**12 operations** (5 CRUD + 7 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get projects paginated | `--startAt`, `--maxResults`, `--orderBy`, `--id`, `--keys`, `--query`, `--typeKey`, `--categoryId`, `--action`, `--expand`, `--status`, `--properties`, `--propertyQuery` (all query) |
| `create` | Create project [body required] | (body only) |
| `get <projectIdOrKey>` | Get project | `--projectIdOrKey` (path, required), `--expand`, `--properties` (query) |
| `update <projectIdOrKey>` | Update project [body required] | `--projectIdOrKey` (path, required), `--expand` (query) |
| `delete <projectIdOrKey>` | Delete project | `--projectIdOrKey` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-recent` | GET | Get recent projects |
| `archive-project` | POST | Archive project |
| `delete-project-asynchronously` | POST | Delete project asynchronously |
| `restore` | POST | Restore deleted or archived project |
| `get-all-statuses` | GET | Get all statuses for project |
| `get-hierarchy` | GET | Get project issue type hierarchy |
| `get-notification-scheme-for-project` | GET | Get project notification scheme |

### Examples

```bash
# List all projects
shrug jira projects list

# List projects filtered by type
shrug jira projects list --typeKey software --maxResults 50

# Get a project by key
shrug jira projects get PROJ

# Get a project with expanded details
shrug jira projects get PROJ --expand description,lead,url

# Create a project
echo '{"key":"NEW","name":"New Project","projectTypeKey":"software","leadAccountId":"..."}' | shrug jira projects create

# Archive a project
shrug jira projects archive-project --projectIdOrKey PROJ

# Restore a deleted project
shrug jira projects restore --projectIdOrKey PROJ

# Get all statuses for a project
shrug jira projects get-all-statuses --projectIdOrKey PROJ
```

---

## dashboards

Manage Jira dashboards: CRUD, gadgets, item properties, copying, and bulk editing.

**17 operations** (5 CRUD + 12 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get all dashboards | `--filter`, `--startAt`, `--maxResults` (all query) |
| `create` | Create dashboard [body required] | (body only) |
| `get <id>` | Get dashboard | `--id` (path, required) |
| `update <id>` | Update dashboard [body required] | `--id` (path, required) |
| `delete <id>` | Delete dashboard | `--id` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `bulk-edit-dashboards` | PUT | Bulk edit dashboards |
| `get-all-available-dashboard-gadgets` | GET | Get available gadgets |
| `get-dashboards-paginated` | GET | Search for dashboards |
| `get-all-gadgets` | GET | Get gadgets |
| `add-gadget` | POST | Add gadget to dashboard |
| `update-gadget` | PUT | Update gadget on dashboard |
| `remove-gadget` | DELETE | Remove gadget from dashboard |
| `get-dashboard-item-property-keys` | GET | Get dashboard item property keys |
| `get-dashboard-item-property` | GET | Get dashboard item property |
| `set-dashboard-item-property` | PUT | Set dashboard item property |
| `delete-dashboard-item-property` | DELETE | Delete dashboard item property |
| `copy-dashboard` | POST | Copy dashboard |

### Examples

```bash
# List all dashboards
shrug jira dashboards list

# Get a dashboard by ID
shrug jira dashboards get 10001

# Search dashboards
shrug jira dashboards get-dashboards-paginated --dashboardName "Sprint"

# Copy a dashboard
shrug jira dashboards copy-dashboard --id 10001

# Get gadgets on a dashboard
shrug jira dashboards get-all-gadgets --dashboardId 10001

# Add a gadget to a dashboard
echo '{"uri":"...","position":{"column":0,"row":0}}' | shrug jira dashboards add-gadget --dashboardId 10001
```

---

## filters

Manage saved Jira filters: CRUD, favourites, columns, ownership, and search.

**13 operations** (5 CRUD + 8 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get my filters | `--expand`, `--includeFavourites` (all query) |
| `create` | Create filter [body required] | (body only) |
| `get <id>` | Get filter | `--id` (path, required), `--expand` (query) |
| `update <id>` | Update filter [body required] | `--id` (path, required) |
| `delete <id>` | Delete filter | `--id` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-favourite-filters` | GET | Get favorite filters |
| `get-filters-paginated` | GET | Search for filters |
| `get-columns` | GET | Get columns |
| `set-columns` | PUT | Set columns |
| `reset-columns` | DELETE | Reset columns |
| `set-favourite-for-filter` | PUT | Add filter as favorite |
| `delete-favourite-for-filter` | DELETE | Remove filter as favorite |
| `change-filter-owner` | PUT | Change filter owner |

### Examples

```bash
# List my filters
shrug jira filters list

# List with favourites included
shrug jira filters list --includeFavourites true

# Get a specific filter
shrug jira filters get 10100

# Search for filters by name
shrug jira filters get-filters-paginated --filterName "Sprint"

# Add a filter to favourites
shrug jira filters set-favourite-for-filter --id 10100

# Change filter owner
echo '{"accountId":"5b10ac8d82e05b22cc7d4ef5"}' | shrug jira filters change-filter-owner --id 10100
```

---

## labels

Read-only label listing.

**1 operation** (1 CRUD)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get all labels | `--startAt`, `--maxResults` (all query) |

### Examples

```bash
# List all labels
shrug jira labels list

# List labels with pagination
shrug jira labels list --startAt 0 --maxResults 100
```

---

## issue search

Search issues using JQL, with enhanced search and issue picker suggestions.

**5 operations** (2 CRUD + 3 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Search for issues using JQL enhanced search (GET) | `--jql`, `--nextPageToken`, `--maxResults`, `--fields`, `--expand`, `--properties`, `--fieldsByKeys`, `--failFast`, `--reconcileIssues` (all query) |
| `create` | Check issues against JQL [body required] | (body only) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-issue-picker-resource` | GET | Get issue picker suggestions |
| `count-issues` | POST | Count issues using JQL |
| `search-and-reconsile-issues-using-jql-post` | POST | Search for issues using JQL enhanced search (POST) |

### Examples

```bash
# Search for open bugs in a project
shrug jira "issue search" list --jql "project = PROJ AND type = Bug AND status != Done"

# Search with specific fields returned
shrug jira "issue search" list --jql "assignee = currentUser()" --fields summary,status,priority

# Count issues matching a JQL query
echo '{"jql":"project = PROJ"}' | shrug jira "issue search" count-issues

# Get issue picker suggestions
shrug jira "issue search" get-issue-picker-resource --query "login bug"
```

---

## jql

JQL field reference data, query parsing, sanitisation, and autocomplete.

**6 operations** (2 CRUD + 4 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get field reference data (GET) | (no parameters) |
| `create` | Parse JQL query [body required] | (body only) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-auto-complete-post` | POST | Get field reference data (POST) |
| `get-field-auto-complete-for-query-string` | GET | Get field auto complete suggestions |
| `migrate-queries` | POST | Convert user identifiers to account IDs in JQL queries |
| `sanitise-jql-queries` | POST | Sanitize JQL queries |

### Examples

```bash
# Get all available JQL fields
shrug jira jql list

# Parse a JQL query for validation
echo '{"queries":["project = PROJ AND status = Open"]}' | shrug jira jql create

# Get autocomplete suggestions
shrug jira jql get-field-auto-complete-for-query-string --fieldName status --fieldValue "In"

# Sanitise JQL queries
echo '{"queries":[{"query":"assignee = bob"}]}' | shrug jira jql sanitise-jql-queries
```

---

## jql functions (apps)

Manage JQL function precomputations for Connect and Forge apps.

**3 operations** (2 CRUD + 1 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get precomputations (apps) | (no parameters) |
| `create` | Update precomputations (apps) [body required] | (body only) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-precomputations-by-i-d` | POST | Get precomputations by ID (apps) |

### Examples

```bash
# List all precomputations
shrug jira "jql functions (apps)" list

# Get specific precomputations by ID
echo '{"ids":["abc-123"]}' | shrug jira "jql functions (apps)" get-precomputations-by-i-d
```
