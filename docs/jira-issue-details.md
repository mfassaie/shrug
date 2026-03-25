# Jira Issue Details

Command reference for issue-level detail entities: comments, attachments, worklogs, votes, watchers, links, remote links, and properties.

All commands follow the pattern:

```
shrug jira "<tag>" <operation> [--param value]
```

---

## issue comments

Manage comments on Jira issues: add, retrieve, update, delete, and bulk fetch by IDs.

**6 operations** (4 CRUD + 2 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `create` | Get comments by IDs [body required] | (body only) |
| `get <issueIdOrKey>` | Get comments | `--issueIdOrKey` (path, required), `--startAt`, `--maxResults`, `--orderBy`, `--expand` (all query) |
| `update <issueIdOrKey>` | Update comment [body required] | `--issueIdOrKey` (path, required), `--notifyUsers`, `--overrideEditableFlag`, `--expand` (query) |
| `delete <issueIdOrKey>` | Delete comment | `--issueIdOrKey` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `add-comment` | POST | Add comment |
| `get-comment` | GET | Get comment |

#### add-comment parameters

| Parameter | Location | Required |
|-----------|----------|----------|
| `--issueIdOrKey` | path | yes |
| `--expand` | query | no |

### Examples

```bash
# Get all comments on an issue
shrug jira "issue comments" get PROJ-123

# Get comments ordered by creation date
shrug jira "issue comments" get PROJ-123 --orderBy created --maxResults 50

# Add a comment to an issue
echo '{"body":{"type":"doc","version":1,"content":[{"type":"paragraph","content":[{"type":"text","text":"This is a comment."}]}]}}' | shrug jira "issue comments" add-comment --issueIdOrKey PROJ-123

# Update a comment
echo '{"body":{"type":"doc","version":1,"content":[{"type":"paragraph","content":[{"type":"text","text":"Updated comment."}]}]}}' | shrug jira "issue comments" update PROJ-123 --id 10001

# Delete a comment
shrug jira "issue comments" delete PROJ-123 --id 10001

# Bulk fetch comments by IDs
echo '{"ids":[10001, 10002, 10003]}' | shrug jira "issue comments" create
```

---

## issue attachments

Manage issue attachments: metadata, content download, thumbnails, and upload.

**8 operations** (3 CRUD + 5 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get Jira attachment settings | (no parameters) |
| `get <id>` | Get attachment metadata | `--id` (path, required) |
| `delete <id>` | Delete attachment | `--id` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-attachment-content` | GET | Get attachment content |
| `get-attachment-thumbnail` | GET | Get attachment thumbnail |
| `expand-attachment-for-humans` | GET | Get all metadata for an expanded attachment |
| `expand-attachment-for-machines` | GET | Get contents metadata for an expanded attachment |
| `add-attachment` | POST | Add attachment |

### Examples

```bash
# Get attachment settings for the instance
shrug jira "issue attachments" list

# Get metadata for an attachment
shrug jira "issue attachments" get 10001

# Download attachment content
shrug jira "issue attachments" get-attachment-content --id 10001

# Get attachment thumbnail
shrug jira "issue attachments" get-attachment-thumbnail --id 10001

# Get expanded metadata (human-readable)
shrug jira "issue attachments" expand-attachment-for-humans --id 10001

# Delete an attachment
shrug jira "issue attachments" delete 10001

# Add an attachment to an issue
shrug jira "issue attachments" add-attachment --issueIdOrKey PROJ-123
```

---

## issue worklogs

Manage time-tracking worklogs: add, update, delete, bulk operations, and change tracking.

**11 operations** (5 CRUD + 6 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get IDs of deleted worklogs | (query parameters) |
| `create` | Get worklogs [body required] | (body only) |
| `get <issueIdOrKey>` | Get issue worklogs | `--issueIdOrKey` (path, required), `--startAt`, `--maxResults`, `--startedAfter`, `--startedBefore`, `--expand` (all query) |
| `update <issueIdOrKey>` | Update worklog [body required] | `--issueIdOrKey` (path, required) |
| `delete <issueIdOrKey>` | Bulk delete worklogs [body required] | `--issueIdOrKey` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `add-worklog` | POST | Add worklog |
| `bulk-move-worklogs` | POST | Bulk move worklogs |
| `get-worklog` | GET | Get worklog |
| `delete-worklog` | DELETE | Delete worklog |
| `get-ids-of-worklogs-modified-since` | GET | Get IDs of updated worklogs |

#### add-worklog parameters

| Parameter | Location | Required |
|-----------|----------|----------|
| `--issueIdOrKey` | path | yes |
| `--notifyUsers` | query | no |
| `--adjustEstimate` | query | no |
| `--newEstimate` | query | no |
| `--reduceBy` | query | no |
| `--expand` | query | no |
| `--overrideEditableFlag` | query | no |

### Examples

```bash
# Get worklogs for an issue
shrug jira "issue worklogs" get PROJ-123

# Get worklogs within a date range
shrug jira "issue worklogs" get PROJ-123 --startedAfter 1617235200000 --startedBefore 1619827200000

# Add a worklog to an issue
echo '{"timeSpentSeconds":3600,"started":"2024-01-15T09:00:00.000+0000"}' | shrug jira "issue worklogs" add-worklog --issueIdOrKey PROJ-123

# Add worklog and adjust remaining estimate
echo '{"timeSpentSeconds":7200}' | shrug jira "issue worklogs" add-worklog --issueIdOrKey PROJ-123 --adjustEstimate auto

# Get IDs of deleted worklogs
shrug jira "issue worklogs" list

# Get IDs of recently modified worklogs
shrug jira "issue worklogs" get-ids-of-worklogs-modified-since --since 1617235200000
```

---

## issue votes

Manage issue votes: view, add, and remove votes.

**3 operations** (2 CRUD + 1 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `get <issueIdOrKey>` | Get votes | `--issueIdOrKey` (path, required) |
| `delete <issueIdOrKey>` | Delete vote | `--issueIdOrKey` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `add-vote` | POST | Add vote |

### Examples

```bash
# Get votes on an issue
shrug jira "issue votes" get PROJ-123

# Vote on an issue
shrug jira "issue votes" add-vote --issueIdOrKey PROJ-123

# Remove your vote
shrug jira "issue votes" delete PROJ-123
```

---

## issue watchers

Manage issue watchers: view, add, remove, and bulk check.

**4 operations** (3 CRUD + 1 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `create` | Get is watching issue bulk [body required] | (body only) |
| `get <issueIdOrKey>` | Get issue watchers | `--issueIdOrKey` (path, required) |
| `delete <issueIdOrKey>` | Delete watcher | `--issueIdOrKey` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `add-watcher` | POST | Add watcher |

### Examples

```bash
# Get watchers for an issue
shrug jira "issue watchers" get PROJ-123

# Add a watcher to an issue
echo '"5b10ac8d82e05b22cc7d4ef5"' | shrug jira "issue watchers" add-watcher --issueIdOrKey PROJ-123

# Remove a watcher
shrug jira "issue watchers" delete PROJ-123 --accountId 5b10ac8d82e05b22cc7d4ef5

# Bulk check if watching issues
echo '{"issueIds":["10001","10002"]}' | shrug jira "issue watchers" create
```

---

## issue links

Create, read, and delete links between issues.

**3 operations** (3 CRUD)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `create` | Create issue link [body required] | (body only) |
| `get <linkId>` | Get issue link | `--linkId` (path, required) |
| `delete <linkId>` | Delete issue link | `--linkId` (path, required) |

### Examples

```bash
# Get an issue link by ID
shrug jira "issue links" get 10001

# Create a link between two issues
echo '{"type":{"name":"Blocks"},"inwardIssue":{"key":"PROJ-123"},"outwardIssue":{"key":"PROJ-456"}}' | shrug jira "issue links" create

# Delete an issue link
shrug jira "issue links" delete 10001
```

---

## issue link types

CRUD for the types of links available between issues (Blocks, Duplicates, etc.).

**5 operations** (5 CRUD)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get issue link types | (no parameters) |
| `create` | Create issue link type [body required] | (body only) |
| `get <issueLinkTypeId>` | Get issue link type | `--issueLinkTypeId` (path, required) |
| `update <issueLinkTypeId>` | Update issue link type [body required] | `--issueLinkTypeId` (path, required) |
| `delete <issueLinkTypeId>` | Delete issue link type | `--issueLinkTypeId` (path, required) |

### Examples

```bash
# List all link types
shrug jira "issue link types" list

# Get a specific link type
shrug jira "issue link types" get 10001

# Create a new link type
echo '{"name":"Caused by","inward":"is caused by","outward":"causes"}' | shrug jira "issue link types" create

# Delete a link type
shrug jira "issue link types" delete 10001
```

---

## issue remote links

Manage remote links on issues (links to external systems).

**6 operations** (3 CRUD + 3 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `get <issueIdOrKey>` | Get remote issue links | `--issueIdOrKey` (path, required) |
| `update <issueIdOrKey>` | Update remote issue link by ID [body required] | `--issueIdOrKey` (path, required) |
| `delete <issueIdOrKey>` | Delete remote issue link by global ID | `--issueIdOrKey` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `create-or-update-remote-issue-link` | POST | Create or update remote issue link |
| `get-remote-issue-link-by-id` | GET | Get remote issue link by ID |
| `delete-remote-issue-link-by-id` | DELETE | Delete remote issue link by ID |

### Examples

```bash
# Get all remote links on an issue
shrug jira "issue remote links" get PROJ-123

# Create a remote link
echo '{"object":{"url":"https://example.com/item/123","title":"External Item"}}' | shrug jira "issue remote links" create-or-update-remote-issue-link --issueIdOrKey PROJ-123

# Get a specific remote link by ID
shrug jira "issue remote links" get-remote-issue-link-by-id --issueIdOrKey PROJ-123 --linkId 10001

# Delete a remote link by ID
shrug jira "issue remote links" delete-remote-issue-link-by-id --issueIdOrKey PROJ-123 --linkId 10001
```

---

## issue properties

Manage arbitrary key-value properties on issues, with bulk set and delete support.

**8 operations** (4 CRUD + 4 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `create` | Bulk set issues properties by list [body required] | (body only) |
| `get <issueIdOrKey>` | Get issue property keys | `--issueIdOrKey` (path, required) |
| `update <issueIdOrKey>` | Bulk set issue property [body required] | `--issueIdOrKey` (path, required) |
| `delete <issueIdOrKey>` | Bulk delete issue property [body required] | `--issueIdOrKey` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `bulk-set-issue-properties-by-issue` | POST | Bulk set issue properties by issue |
| `get-issue-property` | GET | Get issue property |
| `set-issue-property` | PUT | Set issue property |
| `delete-issue-property` | DELETE | Delete issue property |

### Examples

```bash
# Get property keys for an issue
shrug jira "issue properties" get PROJ-123

# Get a specific property value
shrug jira "issue properties" get-issue-property --issueIdOrKey PROJ-123 --propertyKey myapp.data

# Set a property on an issue
echo '{"value":"some data"}' | shrug jira "issue properties" set-issue-property --issueIdOrKey PROJ-123 --propertyKey myapp.data

# Delete a property
shrug jira "issue properties" delete-issue-property --issueIdOrKey PROJ-123 --propertyKey myapp.data

# Bulk set properties across issues
echo '{"entitiesIds":[10001,10002],"properties":{"myapp.flag":"true"}}' | shrug jira "issue properties" create
```
