# Jira Fields

Command reference for issue fields, custom field contexts, custom field options, custom field values, custom field configuration, custom field associations, field configurations, and field schemes.

All commands follow the pattern:

```
shrug jira "<tag>" <operation> [--param value]
```

---

## issue fields

Manage Jira fields: list system and custom fields, create and update custom fields, trash management.

**10 operations** (5 CRUD + 5 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get fields | (no parameters) |
| `create` | Create custom field [body required] | (body only) |
| `get <fieldId>` | Get contexts for a field | `--fieldId` (path, required) |
| `update <fieldId>` | Update custom field [body required] | `--fieldId` (path, required) |
| `delete <fieldId>` | Delete custom field | `--fieldId` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-fields-paginated` | GET | Get fields paginated |
| `get-trashed-fields-paginated` | GET | Get fields in trash paginated |
| `restore-custom-field` | POST | Restore custom field from trash |
| `trash-custom-field` | POST | Move custom field to trash |
| `get-project-fields` | GET | Get fields for projects |

### Examples

```bash
# List all fields (system + custom)
shrug jira "issue fields" list

# Get fields with pagination
shrug jira "issue fields" get-fields-paginated --startAt 0 --maxResults 50

# Create a custom field
echo '{"name":"My Field","type":"com.atlassian.jira.plugin.system.customfieldtypes:textfield","searcherKey":"com.atlassian.jira.plugin.system.customfieldtypes:textsearcher"}' | shrug jira "issue fields" create

# Get contexts for a custom field
shrug jira "issue fields" get customfield_10001

# Move a custom field to trash
shrug jira "issue fields" trash-custom-field --id customfield_10001

# Restore a trashed field
shrug jira "issue fields" restore-custom-field --id customfield_10001

# Get trashed fields
shrug jira "issue fields" get-trashed-fields-paginated

# Get fields available for specific projects
shrug jira "issue fields" get-project-fields --projectIds 10001,10002
```

---

## issue custom field contexts

Manage contexts for custom fields: create, update, delete, and map contexts to projects and issue types.

**11 operations** (3 CRUD + 8 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `get <fieldId>` | Get custom field contexts | `--fieldId` (path, required) |
| `update <fieldId>` | Update custom field context [body required] | `--fieldId` (path, required) |
| `delete <fieldId>` | Delete custom field context | `--fieldId` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `create-custom-field-context` | POST | Create custom field context |
| `get-issue-type-mappings-for-contexts` | GET | Get issue types for custom field context |
| `get-custom-field-contexts-for-projects-and-issue-types` | POST | Get custom field contexts for projects and issue types |
| `get-project-context-mapping` | GET | Get project mappings for custom field context |
| `add-issue-types-to-context` | PUT | Add issue types to context |
| `remove-issue-types-from-context` | POST | Remove issue types from context |
| `assign-projects-to-custom-field-context` | PUT | Assign custom field context to projects |
| `remove-custom-field-context-from-projects` | POST | Remove custom field context from projects |

### Examples

```bash
# Get contexts for a custom field
shrug jira "issue custom field contexts" get customfield_10001

# Create a context for a custom field
echo '{"name":"Bug context","issueTypeIds":["10001"],"projectIds":["10000"]}' | shrug jira "issue custom field contexts" create-custom-field-context --fieldId customfield_10001

# Assign projects to a context
echo '{"projectIds":["10000","10001"]}' | shrug jira "issue custom field contexts" assign-projects-to-custom-field-context --fieldId customfield_10001 --contextId 10100

# Add issue types to a context
echo '{"issueTypeIds":["10001","10002"]}' | shrug jira "issue custom field contexts" add-issue-types-to-context --fieldId customfield_10001 --contextId 10100
```

---

## issue custom field options

Manage the selectable options for custom fields within contexts.

**7 operations** (3 CRUD + 4 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `get <id>` | Get custom field option | `--id` (path, required) |
| `update <id>` | Update custom field options (context) [body required] | `--id` (path, required) |
| `delete <id>` | Delete custom field options (context) | `--id` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-options-for-context` | GET | Get custom field options (context) |
| `create-custom-field-option` | POST | Create custom field options (context) |
| `reorder-custom-field-options` | PUT | Reorder custom field options (context) |
| `replace-custom-field-option` | DELETE | Replace custom field options |

### Examples

```bash
# Get options for a custom field context
shrug jira "issue custom field options" get-options-for-context --fieldId customfield_10001 --contextId 10100

# Create options for a context
echo '{"options":[{"value":"Option A"},{"value":"Option B"}]}' | shrug jira "issue custom field options" create-custom-field-option --fieldId customfield_10001 --contextId 10100

# Reorder options
echo '{"customFieldOptionIds":["10200","10201"],"position":"First"}' | shrug jira "issue custom field options" reorder-custom-field-options --fieldId customfield_10001 --contextId 10100

# Delete an option
shrug jira "issue custom field options" delete 10200
```

---

## issue custom field options (apps)

Manage issue field options for Connect and Forge apps.

**9 operations** (4 CRUD + 5 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get all issue field options | (no parameters) |
| `get <fieldKey>` | Get all issue field options | `--fieldKey` (path, required) |
| `update <fieldKey>` | Update issue field option [body required] | `--fieldKey` (path, required) |
| `delete <fieldKey>` | Delete issue field option | `--fieldKey` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `create-issue-field-option` | POST | Create issue field option |
| `get-selectable-issue-field-options` | GET | Get selectable issue field options |
| `get-visible-issue-field-options` | GET | Get visible issue field options |
| `get-issue-field-option` | GET | Get issue field option |
| `replace-issue-field-option` | DELETE | Replace issue field option |

### Examples

```bash
# Get all options for a field
shrug jira "issue custom field options (apps)" get customfield_10001

# Create an option
echo '{"value":"New Option","properties":{"key":"value"}}' | shrug jira "issue custom field options (apps)" create-issue-field-option --fieldKey customfield_10001

# Get selectable options (respects context)
shrug jira "issue custom field options (apps)" get-selectable-issue-field-options --fieldKey customfield_10001
```

---

## issue custom field values (apps)

Update custom field values for Connect and Forge apps.

**2 operations** (2 CRUD)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `create` | Update custom fields [body required] | (body only) |
| `update <fieldIdOrKey>` | Update custom field value [body required] | `--fieldIdOrKey` (path, required) |

### Examples

```bash
# Bulk update custom field values
echo '{"updates":[{"issueIds":[10001],"value":"new value"}]}' | shrug jira "issue custom field values (apps)" create

# Update a specific field value
echo '{"updates":[{"issueIds":[10001],"value":"updated"}]}' | shrug jira "issue custom field values (apps)" update customfield_10001
```

---

## issue custom field configuration (apps)

Manage custom field configurations for Connect and Forge apps.

**3 operations** (3 CRUD)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `create` | Bulk get custom field configurations [body required] | (body only) |
| `get <fieldIdOrKey>` | Get custom field configurations | `--fieldIdOrKey` (path, required) |
| `update <fieldIdOrKey>` | Update custom field configurations [body required] | `--fieldIdOrKey` (path, required) |

### Examples

```bash
# Get configuration for a custom field
shrug jira "issue custom field configuration (apps)" get customfield_10001

# Update field configuration
echo '{"configurations":[{"contextId":"10100","schema":{"type":"string"}}]}' | shrug jira "issue custom field configuration (apps)" update customfield_10001
```

---

## issue custom field associations

Manage associations between custom fields and screens/projects.

**2 operations** (2 raw)

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `create-associations` | PUT | Create associations |
| `remove-associations` | DELETE | Remove associations |

### Examples

```bash
# Create field associations
echo '{"associations":[{"fieldId":"customfield_10001","screenId":"10100"}]}' | shrug jira "issue custom field associations" create-associations

# Remove field associations
echo '{"associations":[{"fieldId":"customfield_10001","screenId":"10100"}]}' | shrug jira "issue custom field associations" remove-associations
```

---

## issue field configurations

Manage field configurations (which fields appear on create/edit screens and their behaviour).

**2 operations** (2 CRUD)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `get <id>` | Get field configuration items | `--id` (path, required) |
| `delete <id>` | Delete field configuration | `--id` (path, required) |

### Examples

```bash
# Get items in a field configuration
shrug jira "issue field configurations" get 10001

# Delete a field configuration
shrug jira "issue field configurations" delete 10001
```

---

## field schemes

Manage field schemes: CRUD, field associations, project mappings, cloning, and parameter management.

**15 operations** (5 CRUD + 10 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get field schemes | (pagination query parameters) |
| `create` | Create field scheme [body required] | (body only) |
| `get <id>` | Get field scheme | `--id` (path, required) |
| `update <id>` | Update field scheme [body required] | `--id` (path, required) |
| `delete <id>` | Delete a field scheme | `--id` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `update-fields-associated-with-schemes` | PUT | Update fields associated with field schemes |
| `remove-fields-associated-with-schemes` | DELETE | Remove fields associated with field schemes |
| `update-field-association-scheme-item-parameters` | PUT | Update field parameters |
| `remove-field-association-scheme-item-parameters` | DELETE | Remove field parameters |
| `get-projects-with-field-schemes` | GET | Get projects with field schemes |
| `associate-projects-to-field-association-schemes` | PUT | Associate projects to field schemes |
| `clone-field-association-scheme` | POST | Clone field scheme |
| `search-field-association-scheme-fields` | GET | Search field scheme fields |
| `get-field-association-scheme-item-parameters` | GET | Get field parameters |
| `search-field-association-scheme-projects` | GET | Search field scheme projects |

### Examples

```bash
# List all field schemes
shrug jira "field schemes" list

# Get a field scheme by ID
shrug jira "field schemes" get 10001

# Create a new field scheme
echo '{"name":"My Field Scheme","description":"Custom scheme"}' | shrug jira "field schemes" create

# Clone a field scheme
shrug jira "field schemes" clone-field-association-scheme --id 10001

# Associate projects with a field scheme
echo '{"projectIds":["10000","10001"]}' | shrug jira "field schemes" associate-projects-to-field-association-schemes --id 10001

# Search fields in a scheme
shrug jira "field schemes" search-field-association-scheme-fields --id 10001

# Get projects using a field scheme
shrug jira "field schemes" get-projects-with-field-schemes --schemeId 10001
```
