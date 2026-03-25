# Jira Screens

Command reference for screens, screen schemes, screen tabs, and screen tab fields.

All commands follow the pattern:

```
shrug jira "<tag>" <operation> [--param value]
```

---

## screens

Manage Jira screens: CRUD, default screen field management, and available field listing.

**7 operations** (5 CRUD + 2 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get screens | `--startAt`, `--maxResults`, `--id`, `--queryString`, `--scope`, `--orderBy` (all query) |
| `create` | Create screen [body required] | (body only) |
| `get <fieldId>` | Get screens for a field | `--fieldId` (path, required) |
| `update <fieldId>` | Update screen [body required] | `--fieldId` (path, required) |
| `delete <fieldId>` | Delete screen | `--fieldId` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `add-field-to-default-screen` | POST | Add field to default screen |
| `get-available-screen-fields` | GET | Get available screen fields |

### Examples

```bash
# List all screens
shrug jira screens list

# Search screens by name
shrug jira screens list --queryString "Default" --maxResults 20

# Get screens for a specific field
shrug jira screens get customfield_10001

# Create a screen
echo '{"name":"My Screen","description":"Custom screen for bugs"}' | shrug jira screens create

# Get available fields that can be added to a screen
shrug jira screens get-available-screen-fields --screenId 10001

# Add a field to the default screen
shrug jira screens add-field-to-default-screen --fieldId customfield_10001

# Delete a screen
shrug jira screens delete 10001
```

---

## screen schemes

Manage screen schemes: create, update, delete, and list.

**4 operations** (4 CRUD)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get screen schemes | (pagination query parameters) |
| `create` | Create screen scheme [body required] | (body only) |
| `update <screenSchemeId>` | Update screen scheme [body required] | `--screenSchemeId` (path, required) |
| `delete <screenSchemeId>` | Delete screen scheme | `--screenSchemeId` (path, required) |

### Examples

```bash
# List all screen schemes
shrug jira "screen schemes" list

# Create a screen scheme
echo '{"name":"Bug Screen Scheme","screens":{"default":10001,"edit":10002,"create":10003}}' | shrug jira "screen schemes" create

# Update a screen scheme
echo '{"name":"Updated Scheme","screens":{"default":10001}}' | shrug jira "screen schemes" update 10100

# Delete a screen scheme
shrug jira "screen schemes" delete 10100
```

---

## screen tabs

Manage tabs within screens: list, create, update, delete, and reorder.

**6 operations** (4 CRUD + 2 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get bulk screen tabs | (query parameters) |
| `get <screenId>` | Get all screen tabs | `--screenId` (path, required) |
| `update <screenId>` | Update screen tab [body required] | `--screenId` (path, required) |
| `delete <screenId>` | Delete screen tab | `--screenId` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `add-screen-tab` | POST | Create screen tab |
| `move-screen-tab` | POST | Move screen tab |

### Examples

```bash
# Get all tabs for a screen
shrug jira "screen tabs" get 10001

# Bulk get screen tabs
shrug jira "screen tabs" list --screenId 10001,10002

# Create a new tab on a screen
echo '{"name":"Details"}' | shrug jira "screen tabs" add-screen-tab --screenId 10001

# Update a tab name
echo '{"name":"Renamed Tab"}' | shrug jira "screen tabs" update 10001 --tabId 10100

# Move a tab to a different position
shrug jira "screen tabs" move-screen-tab --screenId 10001 --tabId 10100 --pos 0

# Delete a screen tab
shrug jira "screen tabs" delete 10001 --tabId 10100
```

---

## screen tab fields

Manage fields within screen tabs: list, add, remove, and reorder.

**5 operations** (3 CRUD + 2 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get all screen tab fields | (query parameters) |
| `get <screenId>` | Get all screen tab fields | `--screenId` (path, required) |
| `delete <screenId>` | Remove screen tab field | `--screenId` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `add-screen-tab-field` | POST | Add screen tab field |
| `move-screen-tab-field` | POST | Move screen tab field |

### Examples

```bash
# Get fields on a screen tab
shrug jira "screen tab fields" get 10001 --tabId 10100

# List all screen tab fields
shrug jira "screen tab fields" list --screenId 10001 --tabId 10100

# Add a field to a screen tab
echo '{"fieldId":"summary"}' | shrug jira "screen tab fields" add-screen-tab-field --screenId 10001 --tabId 10100

# Move a field to a different position
echo '{"position":"Earlier"}' | shrug jira "screen tab fields" move-screen-tab-field --screenId 10001 --tabId 10100 --id summary

# Remove a field from a screen tab
shrug jira "screen tab fields" delete 10001 --tabId 10100 --id summary
```
