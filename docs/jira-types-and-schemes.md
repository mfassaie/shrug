# Jira Types and Schemes

Command reference for issue types, issue type properties, issue type schemes, issue type screen schemes, notification schemes, security schemes, security levels, and priority schemes.

All commands follow the pattern:

```
shrug jira "<tag>" <operation> [--param value]
```

---

## issue types

Manage Jira issue types: CRUD, project-scoped listing, alternatives, and avatar upload.

**8 operations** (5 CRUD + 3 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get all issue types for user | (no parameters) |
| `create` | Create issue type [body required] | (body only) |
| `get <id>` | Get issue type | `--id` (path, required) |
| `update <id>` | Update issue type [body required] | `--id` (path, required) |
| `delete <id>` | Delete issue type | `--id` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-issue-types-for-project` | GET | Get issue types for project |
| `get-alternative-issue-types` | GET | Get alternative issue types |
| `create-issue-type-avatar` | POST | Load issue type avatar |

### Examples

```bash
# List all issue types
shrug jira "issue types" list

# Get a specific issue type
shrug jira "issue types" get 10001

# Get issue types for a project
shrug jira "issue types" get-issue-types-for-project --projectId 10000

# Create an issue type
echo '{"name":"Incident","description":"Production incidents","type":"standard"}' | shrug jira "issue types" create

# Update an issue type
echo '{"name":"Incident (Updated)","description":"Updated description"}' | shrug jira "issue types" update 10001

# Get alternative issue types (for migration)
shrug jira "issue types" get-alternative-issue-types --id 10001

# Delete an issue type
shrug jira "issue types" delete 10001
```

---

## issue type properties

Manage key-value properties on issue types.

**4 operations** (3 CRUD + 1 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `get <issueTypeId>` | Get issue type property keys | `--issueTypeId` (path, required) |
| `update <issueTypeId>` | Set issue type property [body required] | `--issueTypeId` (path, required) |
| `delete <issueTypeId>` | Delete issue type property | `--issueTypeId` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-issue-type-property` | GET | Get issue type property |

### Examples

```bash
# Get property keys for an issue type
shrug jira "issue type properties" get 10001

# Get a specific property
shrug jira "issue type properties" get-issue-type-property --issueTypeId 10001 --propertyKey myapp.config

# Set a property
echo '"my-value"' | shrug jira "issue type properties" update 10001 --propertyKey myapp.config

# Delete a property
shrug jira "issue type properties" delete 10001 --propertyKey myapp.config
```

---

## issue type schemes

Manage issue type schemes: CRUD, issue type mappings, reordering, and project assignment.

**10 operations** (4 CRUD + 6 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get all issue type schemes | (pagination query parameters) |
| `create` | Create issue type scheme [body required] | (body only) |
| `update <issueTypeSchemeId>` | Update issue type scheme [body required] | `--issueTypeSchemeId` (path, required) |
| `delete <issueTypeSchemeId>` | Delete issue type scheme | `--issueTypeSchemeId` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-issue-type-schemes-mapping` | GET | Get issue type scheme items |
| `get-issue-type-scheme-for-projects` | GET | Get issue type schemes for projects |
| `assign-issue-type-scheme-to-project` | PUT | Assign issue type scheme to project |
| `add-issue-types-to-issue-type-scheme` | PUT | Add issue types to issue type scheme |
| `reorder-issue-types-in-issue-type-scheme` | PUT | Change order of issue types |
| `remove-issue-type-from-issue-type-scheme` | DELETE | Remove issue type from issue type scheme |

### Examples

```bash
# List all issue type schemes
shrug jira "issue type schemes" list

# Get issue type scheme items (mappings)
shrug jira "issue type schemes" get-issue-type-schemes-mapping --startAt 0 --maxResults 50

# Get schemes for specific projects
shrug jira "issue type schemes" get-issue-type-scheme-for-projects --projectId 10000,10001

# Create an issue type scheme
echo '{"name":"Software Scheme","issueTypeIds":["10001","10002","10003"],"defaultIssueTypeId":"10001"}' | shrug jira "issue type schemes" create

# Assign a scheme to a project
echo '{"issueTypeSchemeId":"10100","projectId":"10000"}' | shrug jira "issue type schemes" assign-issue-type-scheme-to-project

# Add issue types to a scheme
echo '{"issueTypeIds":["10004"]}' | shrug jira "issue type schemes" add-issue-types-to-issue-type-scheme --issueTypeSchemeId 10100

# Reorder issue types
echo '{"issueTypeIds":["10001","10003","10002"],"position":"First"}' | shrug jira "issue type schemes" reorder-issue-types-in-issue-type-scheme --issueTypeSchemeId 10100

# Remove an issue type from a scheme
shrug jira "issue type schemes" remove-issue-type-from-issue-type-scheme --issueTypeSchemeId 10100 --issueTypeId 10004
```

---

## issue type screen schemes

**Note:** This tag currently triggers a panic in shrug (formatting argument out of range in `src/cmd/crud.rs:248`). This is a known bug. The tag is present in the Jira OpenAPI spec but cannot be enumerated at this time.

Expected operations (based on the Jira API) would include managing the mapping between issue types and screen schemes.

---

## issue notification schemes

Manage notification schemes: CRUD, project mappings, and notification entries.

**8 operations** (5 CRUD + 3 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get notification schemes paginated | (pagination query parameters) |
| `create` | Create notification scheme [body required] | (body only) |
| `get <id>` | Get notification scheme | `--id` (path, required) |
| `update <id>` | Update notification scheme [body required] | `--id` (path, required) |
| `delete <id>` | Delete notification scheme | `--id` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-notification-scheme-to-project-mappings` | GET | Get projects using notification schemes paginated |
| `add-notifications` | PUT | Add notifications to notification scheme |
| `remove-notification-from-notification-scheme` | DELETE | Remove notification from notification scheme |

### Examples

```bash
# List all notification schemes
shrug jira "issue notification schemes" list

# Get a notification scheme
shrug jira "issue notification schemes" get 10001

# Create a notification scheme
echo '{"name":"Custom Notifications","description":"For software projects"}' | shrug jira "issue notification schemes" create

# Get projects using a notification scheme
shrug jira "issue notification schemes" get-notification-scheme-to-project-mappings --schemeId 10001

# Add notifications to a scheme
echo '{"notificationSchemeEvents":[{"event":{"id":"1"},"notifications":[{"notificationType":"CurrentAssignee"}]}]}' | shrug jira "issue notification schemes" add-notifications --id 10001

# Remove a notification from a scheme
shrug jira "issue notification schemes" remove-notification-from-notification-scheme --notificationSchemeId 10001 --notificationId 10200

# Delete a notification scheme
shrug jira "issue notification schemes" delete 10001
```

---

## issue security schemes

Manage issue security schemes: CRUD, security levels, level members, project associations, and defaults.

**17 operations** (5 CRUD + 12 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Search issue security schemes | (pagination query parameters) |
| `create` | Create issue security scheme [body required] | (body only) |
| `get <id>` | Get issue security scheme | `--id` (path, required) |
| `update <id>` | Update issue security scheme [body required] | `--id` (path, required) |
| `delete <id>` | Delete issue security scheme | `--id` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-issue-security-schemes` | GET | Get issue security schemes |
| `get-security-levels` | GET | Get issue security levels |
| `set-default-levels` | PUT | Set default issue security levels |
| `get-security-level-members` | GET | Get issue security level members |
| `search-projects-using-security-schemes` | GET | Get projects using issue security schemes |
| `associate-schemes-to-projects` | PUT | Associate security scheme to project |
| `add-security-level` | PUT | Add issue security levels |
| `update-security-level` | PUT | Update issue security level |
| `remove-level` | DELETE | Remove issue security level |
| `add-security-level-members` | PUT | Add issue security level members |
| `remove-member-from-security-level` | DELETE | Remove member from issue security level |

### Examples

```bash
# List all issue security schemes
shrug jira "issue security schemes" list

# Get a specific security scheme
shrug jira "issue security schemes" get 10001

# Get security levels in a scheme
shrug jira "issue security schemes" get-security-levels --issueSecuritySchemeId 10001

# Get members of a security level
shrug jira "issue security schemes" get-security-level-members --issueSecuritySchemeId 10001 --issueSecurityLevelId 10100

# Create a security scheme
echo '{"name":"Confidential Scheme","description":"Restricts visibility"}' | shrug jira "issue security schemes" create

# Add a security level
echo '{"securityLevels":[{"name":"Internal","description":"Visible to team only"}]}' | shrug jira "issue security schemes" add-security-level --issueSecuritySchemeId 10001

# Associate scheme to a project
echo '{"schemeId":"10001","projectId":"10000"}' | shrug jira "issue security schemes" associate-schemes-to-projects

# Set default security levels
echo '{"defaultValues":[{"issueSecuritySchemeId":"10001","levelId":"10100"}]}' | shrug jira "issue security schemes" set-default-levels
```

---

## issue security level

Read-only access to individual security levels and their members.

**2 operations** (1 CRUD + 1 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `get <id>` | Get issue security level | `--id` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-issue-security-level-members` | GET | Get issue security level members by issue security scheme |

### Examples

```bash
# Get a security level
shrug jira "issue security level" get 10100

# Get members of a security level by scheme
shrug jira "issue security level" get-issue-security-level-members --issueSecuritySchemeId 10001 --issueSecurityLevelId 10100
```

---

## issue security scheme

This tag has no operations available. The functionality is covered by the "issue security schemes" tag above.

---

## priority schemes

Manage priority schemes: CRUD, priority listings, suggestions, and project mappings.

**8 operations** (5 CRUD + 3 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get priority schemes | (pagination query parameters) |
| `create` | Create priority scheme [body required] | (body only) |
| `get <schemeId>` | Get projects by priority scheme | `--schemeId` (path, required) |
| `update <schemeId>` | Update priority scheme [body required] | `--schemeId` (path, required) |
| `delete <schemeId>` | Delete priority scheme | `--schemeId` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `suggested-priorities-for-mappings` | POST | Suggested priorities for mappings |
| `get-available-priorities-by-priority-scheme` | GET | Get available priorities by priority scheme |
| `get-priorities-by-priority-scheme` | GET | Get priorities by priority scheme |

### Examples

```bash
# List all priority schemes
shrug jira "priority schemes" list

# Get projects using a priority scheme
shrug jira "priority schemes" get 10001

# Get priorities in a scheme
shrug jira "priority schemes" get-priorities-by-priority-scheme --schemeId 10001

# Get available priorities for a scheme
shrug jira "priority schemes" get-available-priorities-by-priority-scheme --schemeId 10001

# Create a priority scheme
echo '{"name":"Simple Priorities","description":"Low/Medium/High only"}' | shrug jira "priority schemes" create

# Delete a priority scheme
shrug jira "priority schemes" delete 10001
```
