# Jira Administration

Command reference for administrative tags: application roles, avatars, announcement banner, app data policies, app migration, app properties, audit records, classification levels, dynamic modules, filter sharing, Jira expressions, Jira settings, license metrics, migration of Connect modules to Forge, plans, server info, service registry, status, tasks, teams in plan, time tracking, UI modifications, and webhooks.

All commands follow the pattern:

```
shrug jira "<tag>" <operation> [--param value]
```

---

## application roles

Read-only access to application roles.

**2 operations** (2 CRUD)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get all application roles | (no parameters) |
| `get <key>` | Get application role | `--key` (path, required) |

### Examples

```bash
# List all application roles
shrug jira "application roles" list

# Get a specific role
shrug jira "application roles" get jira-software
```

---

## avatars

Manage system and custom avatars: list, load, delete, and retrieve images.

**8 operations** (3 CRUD + 5 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get system avatars by type | (query parameters) |
| `get <type>` | Get system avatars by type | `--type` (path, required) |
| `delete <type>` | Delete avatar | `--type` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-avatars` | GET | Get avatars |
| `store-avatar` | POST | Load avatar |
| `get-avatar-image-by-type` | GET | Get avatar image by type |
| `get-avatar-image-by-i-d` | GET | Get avatar image by ID |
| `get-avatar-image-by-owner` | GET | Get avatar image by owner |

### Examples

```bash
# Get system avatars for projects
shrug jira avatars get project

# Get avatars for an entity
shrug jira avatars get-avatars --type project --entityId 10000

# Get an avatar image by ID
shrug jira avatars get-avatar-image-by-i-d --type project --id 10100

# Upload an avatar
shrug jira avatars store-avatar --type project

# Delete a custom avatar
shrug jira avatars delete project --id 10100
```

---

## announcement banner

Get and update the Jira announcement banner.

**2 operations** (1 CRUD + 1 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get announcement banner configuration | (no parameters) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `set-banner` | PUT | Update announcement banner configuration |

### Examples

```bash
# Get the current banner
shrug jira "announcement banner" list

# Set the announcement banner
echo '{"message":"System maintenance tonight at 22:00","isDismissible":true,"isEnabled":true}' | shrug jira "announcement banner" set-banner
```

---

## app data policies

Query data policy settings for the workspace and projects.

**2 operations** (1 CRUD + 1 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get data policy for the workspace | (no parameters) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-policies` | GET | Get data policy for projects |

### Examples

```bash
# Get workspace data policy
shrug jira "app data policies" list

# Get data policies for projects
shrug jira "app data policies" get-policies --ids 10000,10001
```

---

## app migration

Manage app migration between Connect and Forge: transition rules and bulk entity updates.

**3 operations** (2 CRUD + 1 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `create` | Get workflow transition rule configurations [body required] | (body only) |
| `update <entityType>` | Bulk update entity properties [body required] | `--entityType` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `app-issue-field-value-update-resource.update-issue-fields_put` | PUT | Bulk update custom field value |

### Examples

```bash
# Get workflow transition rule configs for migration
echo '{"connectWorkflowTransitionRules":[...]}' | shrug jira "app migration" create

# Bulk update entity properties
echo '{"entities":[...]}' | shrug jira "app migration" update IssueProperty
```

---

## app properties

Manage app properties for both Forge and Connect apps.

**8 operations** (4 CRUD + 4 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get app property keys (Forge) | (no parameters) |
| `get <propertyKey>` | Get app property (Forge) | `--propertyKey` (path, required) |
| `update <propertyKey>` | Set app property (Forge) [body required] | `--propertyKey` (path, required) |
| `delete <propertyKey>` | Delete app property (Forge) | `--propertyKey` (path, required) |

### Raw operations (Connect)

| Operation | Method | Description |
|-----------|--------|-------------|
| `addon-properties-resource.get-addon-properties_get` | GET | Get app properties |
| `addon-properties-resource.get-addon-property_get` | GET | Get app property |
| `addon-properties-resource.put-addon-property_put` | PUT | Set app property |
| `addon-properties-resource.delete-addon-property_delete` | DELETE | Delete app property |

### Examples

```bash
# List app property keys (Forge)
shrug jira "app properties" list

# Get a property (Forge)
shrug jira "app properties" get myapp.config

# Set a property (Forge)
echo '{"setting":"value"}' | shrug jira "app properties" update myapp.config

# Delete a property (Forge)
shrug jira "app properties" delete myapp.config

# Get properties (Connect)
shrug jira "app properties" addon-properties-resource.get-addon-properties_get --addonKey my-addon
```

---

## audit records

Read-only access to Jira audit log records.

**1 operation** (1 CRUD)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get audit records | `--offset`, `--limit`, `--filter`, `--from`, `--to` (all query) |

### Examples

```bash
# Get audit records
shrug jira "audit records" list

# Get audit records with filters
shrug jira "audit records" list --filter "User" --from 2024-01-01 --to 2024-01-31

# Paginated audit records
shrug jira "audit records" list --offset 0 --limit 100
```

---

## classification levels

Read-only access to data classification levels configured for the instance.

**1 operation** (1 CRUD)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get all classification levels | (no parameters) |

### Examples

```bash
# List all classification levels
shrug jira "classification levels" list
```

---

## dynamic modules

Manage dynamic modules for Connect apps: list, register, and remove.

**3 operations** (2 CRUD + 1 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get modules | (no parameters) |
| `create` | Register modules [body required] | (body only) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `dynamic-modules-resource.remove-modules_delete` | DELETE | Remove modules |

### Examples

```bash
# List registered modules
shrug jira "dynamic modules" list

# Register new modules
echo '{"modules":{"jiraEntityProperties":[...]}}' | shrug jira "dynamic modules" create

# Remove modules
shrug jira "dynamic modules" dynamic-modules-resource.remove-modules_delete --moduleKey my-module
```

---

## filter sharing

Manage filter share permissions and default share scope.

**6 operations** (3 CRUD + 3 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get default share scope | (no parameters) |
| `get <id>` | Get share permissions | `--id` (path, required) |
| `delete <id>` | Delete share permission | `--id` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `set-default-share-scope` | PUT | Set default share scope |
| `add-share-permission` | POST | Add share permission |
| `get-share-permission` | GET | Get share permission |

### Examples

```bash
# Get the default share scope
shrug jira "filter sharing" list

# Set the default share scope
echo '{"scope":"AUTHENTICATED"}' | shrug jira "filter sharing" set-default-share-scope

# Get share permissions for a filter
shrug jira "filter sharing" get 10100

# Add a share permission
echo '{"type":"group","groupname":"developers"}' | shrug jira "filter sharing" add-share-permission --id 10100

# Get a specific share permission
shrug jira "filter sharing" get-share-permission --id 10100 --permissionId 10200

# Delete a share permission
shrug jira "filter sharing" delete 10100 --permissionId 10200
```

---

## jira expressions

Analyse and evaluate Jira expressions.

**2 operations** (1 CRUD + 1 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `create` | Analyse Jira expression [body required] | (body only) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `evaluate-j-s-i-s-jira-expression` | POST | Evaluate Jira expression using enhanced search API |

### Examples

```bash
# Analyse a Jira expression
echo '{"expressions":["issue.summary"]}' | shrug jira "jira expressions" create

# Evaluate a Jira expression
echo '{"expression":"issue.summary","context":{"issue":{"key":"PROJ-123"}}}' | shrug jira "jira expressions" evaluate-j-s-i-s-jira-expression
```

---

## jira settings

Get and update Jira application settings.

**4 operations** (2 CRUD + 2 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get global settings | (no parameters) |
| `update <id>` | Set application property [body required] | `--id` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-application-property` | GET | Get application property |
| `get-advanced-settings` | GET | Get advanced settings |

### Examples

```bash
# Get global settings
shrug jira "jira settings" list

# Get advanced settings
shrug jira "jira settings" get-advanced-settings

# Get a specific application property
shrug jira "jira settings" get-application-property --key jira.title

# Set an application property
echo '"My Jira Instance"' | shrug jira "jira settings" update jira.title
```

---

## license metrics

Query license information and user counts.

**3 operations** (2 CRUD + 1 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get license | (no parameters) |
| `get <applicationKey>` | Get approximate application license count | `--applicationKey` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-approximate-license-count` | GET | Get approximate license count |

### Examples

```bash
# Get license info
shrug jira "license metrics" list

# Get license count for an application
shrug jira "license metrics" get jira-software

# Get approximate total license count
shrug jira "license metrics" get-approximate-license-count
```

---

## migration of connect modules to forge

Manage migration tasks for Connect to Forge field migrations.

**2 operations** (1 CRUD + 1 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `get <connectKey>` | Get Connect issue field migration task | `--connectKey` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `connect-to-forge-migration-task-submission-resource.submit-task_post` | POST | Submit Connect issue field migration task |

### Examples

```bash
# Get migration task status
shrug jira "migration of connect modules to forge" get my-connect-addon

# Submit a migration task
echo '{"fieldMappings":[...]}' | shrug jira "migration of connect modules to forge" connect-to-forge-migration-task-submission-resource.submit-task_post
```

---

## plans

Manage Jira Plans (Advanced Roadmaps): CRUD, archiving, duplication, and trash.

**7 operations** (4 CRUD + 3 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get plans paginated | (pagination query parameters) |
| `create` | Create plan [body required] | (body only) |
| `get <planId>` | Get plan | `--planId` (path, required) |
| `update <planId>` | Update plan [body required] | `--planId` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `archive-plan` | PUT | Archive plan |
| `duplicate-plan` | POST | Duplicate plan |
| `trash-plan` | PUT | Trash plan |

### Examples

```bash
# List all plans
shrug jira plans list

# Get a specific plan
shrug jira plans get 1

# Create a plan
echo '{"name":"Q2 Roadmap","issueSources":[{"type":"Project","value":"10000"}]}' | shrug jira plans create

# Update a plan
echo '{"name":"Q2 Roadmap (Updated)"}' | shrug jira plans update 1

# Duplicate a plan
shrug jira plans duplicate-plan --planId 1

# Archive a plan
shrug jira plans archive-plan --planId 1

# Trash a plan
shrug jira plans trash-plan --planId 1
```

---

## server info

Get Jira instance information.

**1 operation** (1 CRUD)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get Jira instance info | (no parameters) |

### Examples

```bash
# Get server info
shrug jira "server info" list
```

---

## service registry

Query service registry attributes.

**1 operation** (1 CRUD)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Retrieve the attributes of service registries | (no parameters) |

### Examples

```bash
# Get service registry attributes
shrug jira "service registry" list
```

---

## status

Manage Jira statuses: search, bulk CRUD, name lookup, and usage queries.

**9 operations** (3 CRUD + 6 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Search statuses paginated | (pagination query parameters) |
| `create` | Bulk create statuses [body required] | (body only) |
| `get <statusId>` | Get project usages by status | `--statusId` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-statuses-by-id` | GET | Bulk get statuses |
| `update-statuses` | PUT | Bulk update statuses |
| `delete-statuses-by-id` | DELETE | Bulk delete Statuses |
| `get-statuses-by-name` | GET | Bulk get statuses by name |
| `get-project-issue-type-usages-for-status` | GET | Get issue type usages by status and project |
| `get-workflow-usages-for-status` | GET | Get workflow usages by status |

### Examples

```bash
# Search statuses
shrug jira status list

# Get project usages for a status
shrug jira status get 10001

# Bulk get statuses by ID
shrug jira status get-statuses-by-id --id 1,3,10001

# Bulk get statuses by name
shrug jira status get-statuses-by-name --statusName "In Progress","Done"

# Create statuses in bulk
echo '{"statuses":[{"name":"Reviewing","statusCategory":"IN_PROGRESS","scope":{"type":"PROJECT","project":{"id":"10000"}}}]}' | shrug jira status create

# Bulk update statuses
echo '{"statuses":[{"id":"10001","name":"Under Review"}]}' | shrug jira status update-statuses

# Bulk delete statuses
shrug jira status delete-statuses-by-id --id 10001,10002

# Get workflow usages for a status
shrug jira status get-workflow-usages-for-status --statusId 10001
```

---

## tasks

Get task status and cancel long-running tasks.

**2 operations** (1 CRUD + 1 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `get <taskId>` | Get task | `--taskId` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `cancel-task` | POST | Cancel task |

### Examples

```bash
# Get task progress
shrug jira tasks get 10001

# Cancel a running task
shrug jira tasks cancel-task --taskId 10001
```

---

## teams in plan

Manage teams within Jira Plans: Atlassian teams and plan-only teams.

**9 operations** (3 CRUD + 6 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `get <planId>` | Get teams in plan paginated | `--planId` (path, required) |
| `update <planId>` | Update plan-only team [body required] | `--planId` (path, required) |
| `delete <planId>` | Delete plan-only team | `--planId` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `add-atlassian-team` | POST | Add Atlassian team to plan |
| `get-atlassian-team` | GET | Get Atlassian team in plan |
| `update-atlassian-team` | PUT | Update Atlassian team in plan |
| `remove-atlassian-team` | DELETE | Remove Atlassian team from plan |
| `create-plan-only-team` | POST | Create plan-only team |
| `get-plan-only-team` | GET | Get plan-only team |

### Examples

```bash
# Get teams in a plan
shrug jira "teams in plan" get 1

# Add an Atlassian team to a plan
echo '{"teamId":"team-uuid","planningStyle":"Scrum","velocity":20}' | shrug jira "teams in plan" add-atlassian-team --planId 1

# Get an Atlassian team in a plan
shrug jira "teams in plan" get-atlassian-team --planId 1 --atlassianTeamId team-uuid

# Create a plan-only team
echo '{"name":"Temp Sprint Team","planningStyle":"Scrum","velocity":15}' | shrug jira "teams in plan" create-plan-only-team --planId 1

# Get a plan-only team
shrug jira "teams in plan" get-plan-only-team --planId 1 --planOnlyTeamId 100

# Update a plan-only team
echo '{"name":"Updated Team","velocity":25}' | shrug jira "teams in plan" update 1 --planOnlyTeamId 100

# Remove an Atlassian team from a plan
shrug jira "teams in plan" remove-atlassian-team --planId 1 --atlassianTeamId team-uuid

# Delete a plan-only team
shrug jira "teams in plan" delete 1 --planOnlyTeamId 100
```

---

## time tracking

Manage time tracking providers and settings.

**5 operations** (1 CRUD + 4 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get selected time tracking provider | (no parameters) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `select-time-tracking-implementation` | PUT | Select time tracking provider |
| `get-available-time-tracking-implementations` | GET | Get all time tracking providers |
| `get-shared-time-tracking-configuration` | GET | Get time tracking settings |
| `set-shared-time-tracking-configuration` | PUT | Set time tracking settings |

### Examples

```bash
# Get the selected time tracking provider
shrug jira "time tracking" list

# Get all available providers
shrug jira "time tracking" get-available-time-tracking-implementations

# Get time tracking settings
shrug jira "time tracking" get-shared-time-tracking-configuration

# Set time tracking settings
echo '{"workingHoursPerDay":8,"workingDaysPerWeek":5,"timeFormat":"pretty","defaultUnit":"hour"}' | shrug jira "time tracking" set-shared-time-tracking-configuration

# Select a time tracking provider
echo '{"key":"JIRA"}' | shrug jira "time tracking" select-time-tracking-implementation
```

---

## ui modifications (apps)

Manage UI modifications for Forge apps.

**4 operations** (4 CRUD)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get UI modifications | (no parameters) |
| `create` | Create UI modification [body required] | (body only) |
| `update <uiModificationId>` | Update UI modification [body required] | `--uiModificationId` (path, required) |
| `delete <uiModificationId>` | Delete UI modification | `--uiModificationId` (path, required) |

### Examples

```bash
# List UI modifications
shrug jira "ui modifications (apps)" list

# Create a UI modification
echo '{"name":"Hide Priority","data":"{}","contexts":[{"projectId":"10000","issueTypeId":"10001","viewType":"GIC"}]}' | shrug jira "ui modifications (apps)" create

# Update a UI modification
echo '{"name":"Hide Priority (Updated)","data":"{}"}' | shrug jira "ui modifications (apps)" update 10001

# Delete a UI modification
shrug jira "ui modifications (apps)" delete 10001
```

---

## webhooks

Manage dynamic webhooks for apps: register, list, delete, refresh, and check failures.

**5 operations** (2 CRUD + 3 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get dynamic webhooks for app | (pagination query parameters) |
| `create` | Register dynamic webhooks [body required] | (body only) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `delete-webhook-by-id` | DELETE | Delete webhooks by ID |
| `get-failed-webhooks` | GET | Get failed webhooks |
| `refresh-webhooks` | PUT | Extend webhook life |

### Examples

```bash
# List registered webhooks
shrug jira webhooks list

# Register a webhook
echo '{"webhooks":[{"jqlFilter":"project = PROJ","events":["jira:issue_created","jira:issue_updated"],"url":"https://example.com/webhook"}]}' | shrug jira webhooks create

# Get failed webhooks
shrug jira webhooks get-failed-webhooks

# Refresh (extend) webhook expiry
echo '{"webhookIds":[10001,10002]}' | shrug jira webhooks refresh-webhooks

# Delete a webhook
shrug jira webhooks delete-webhook-by-id --webhookId 10001
```
