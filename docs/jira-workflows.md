# Jira Workflows

Command reference for workflows, workflow schemes, workflow scheme drafts, project associations, statuses, status categories, transition properties, and transition rules.

All commands follow the pattern:

```
shrug jira "<tag>" <operation> [--param value]
```

---

## workflows

Manage Jira workflows: search, bulk CRUD, history, capabilities, previews, and usage queries.

**15 operations** (4 CRUD + 11 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Search workflows | `--startAt`, `--maxResults`, `--expand`, `--queryString`, `--orderBy`, `--scope`, `--isActive` (all query) |
| `create` | Bulk get workflows [body required] | (body only) |
| `get <workflowId>` | Get projects using a given workflow | `--workflowId` (path, required) |
| `delete <workflowId>` | Delete inactive workflow | `--workflowId` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `read-workflow-from-history` | POST | Read workflow version from history |
| `list-workflow-history` | POST | List workflow history entries |
| `get-workflow-project-issue-type-usages` | GET | Get issue types in a project that are using a given workflow |
| `get-workflow-scheme-usages-for-workflow` | GET | Get workflow schemes which are using a given workflow |
| `workflow-capabilities` | GET | Get available workflow capabilities |
| `create-workflows` | POST | Bulk create workflows |
| `validate-create-workflows` | POST | Validate create workflows |
| `get-default-editor` | GET | Get the user's default workflow editor |
| `read-workflow-previews` | POST | Preview workflow |
| `update-workflows` | POST | Bulk update workflows |
| `validate-update-workflows` | POST | Validate update workflows |

### Examples

```bash
# Search all workflows
shrug jira workflows list

# Search workflows by name
shrug jira workflows list --queryString "Bug" --maxResults 20

# Search active workflows only
shrug jira workflows list --isActive true

# Get projects using a workflow
shrug jira workflows get 10001

# Get workflow scheme usages
shrug jira workflows get-workflow-scheme-usages-for-workflow --workflowId 10001

# Get workflow capabilities
shrug jira workflows workflow-capabilities --workflowId 10001

# Delete an inactive workflow
shrug jira workflows delete 10001

# Validate a workflow before creating
echo '{"workflows":[{"name":"New Workflow","statuses":[...]}]}' | shrug jira workflows validate-create-workflows
```

---

## workflow schemes

Manage workflow schemes: CRUD, issue type mappings, default workflow, and project associations.

**19 operations** (5 CRUD + 14 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get all workflow schemes | (pagination query parameters) |
| `create` | Create workflow scheme [body required] | (body only) |
| `get <id>` | Get workflow scheme | `--id` (path, required) |
| `update <id>` | Classic update workflow scheme [body required] | `--id` (path, required) |
| `delete <id>` | Delete workflow scheme | `--id` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `switch-workflow-scheme-for-project` | POST | Switch workflow scheme for project |
| `read-workflow-schemes` | POST | Bulk get workflow schemes |
| `update-schemes` | POST | Update workflow scheme |
| `get-required-workflow-scheme-mappings` | POST | Get required status mappings for workflow scheme update |
| `get-default-workflow` | GET | Get default workflow |
| `update-default-workflow` | PUT | Update default workflow |
| `delete-default-workflow` | DELETE | Delete default workflow |
| `get-workflow-scheme-issue-type` | GET | Get workflow for issue type in workflow scheme |
| `set-workflow-scheme-issue-type` | PUT | Set workflow for issue type in workflow scheme |
| `delete-workflow-scheme-issue-type` | DELETE | Delete workflow for issue type in workflow scheme |
| `get-workflow` | GET | Get issue types for workflows in workflow scheme |
| `update-workflow-mapping` | PUT | Set issue types for workflow in workflow scheme |
| `delete-workflow-mapping` | DELETE | Delete issue types for workflow in workflow scheme |
| `get-project-usages-for-workflow-scheme` | GET | Get projects which are using a given workflow scheme |

### Examples

```bash
# List all workflow schemes
shrug jira "workflow schemes" list

# Get a workflow scheme
shrug jira "workflow schemes" get 10001

# Create a workflow scheme
echo '{"name":"Custom Scheme","description":"For software projects","defaultWorkflow":"jira"}' | shrug jira "workflow schemes" create

# Get the default workflow for a scheme
shrug jira "workflow schemes" get-default-workflow --id 10001

# Set workflow for an issue type in a scheme
echo '{"workflow":"jira","updateDraftIfNeeded":true}' | shrug jira "workflow schemes" set-workflow-scheme-issue-type --id 10001 --issueType 10100

# Get projects using a workflow scheme
shrug jira "workflow schemes" get-project-usages-for-workflow-scheme --workflowSchemeId 10001

# Switch a project's workflow scheme
echo '{"workflowSchemeId":"10002","projectId":"10000"}' | shrug jira "workflow schemes" switch-workflow-scheme-for-project
```

---

## workflow scheme drafts

Manage draft versions of workflow schemes: CRUD, default workflow, issue type mappings, and publishing.

**14 operations** (3 CRUD + 11 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `get <id>` | Get draft workflow scheme | `--id` (path, required) |
| `update <id>` | Update draft workflow scheme [body required] | `--id` (path, required) |
| `delete <id>` | Delete draft workflow scheme | `--id` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `create-workflow-scheme-draft-from-parent` | POST | Create draft workflow scheme |
| `get-draft-default-workflow` | GET | Get draft default workflow |
| `update-draft-default-workflow` | PUT | Update draft default workflow |
| `delete-draft-default-workflow` | DELETE | Delete draft default workflow |
| `get-workflow-scheme-draft-issue-type` | GET | Get workflow for issue type in draft workflow scheme |
| `set-workflow-scheme-draft-issue-type` | PUT | Set workflow for issue type in draft workflow scheme |
| `delete-workflow-scheme-draft-issue-type` | DELETE | Delete workflow for issue type in draft workflow scheme |
| `publish-draft-workflow-scheme` | POST | Publish draft workflow scheme |
| `get-draft-workflow` | GET | Get issue types for workflows in draft workflow scheme |
| `update-draft-workflow-mapping` | PUT | Set issue types for workflow in workflow scheme |
| `delete-draft-workflow-mapping` | DELETE | Delete issue types for workflow in draft workflow scheme |

### Examples

```bash
# Get a draft workflow scheme
shrug jira "workflow scheme drafts" get 10001

# Create a draft from an existing scheme
shrug jira "workflow scheme drafts" create-workflow-scheme-draft-from-parent --id 10001

# Update the draft default workflow
echo '{"workflow":"jira","updateDraftIfNeeded":false}' | shrug jira "workflow scheme drafts" update-draft-default-workflow --id 10001

# Publish the draft
echo '{"statusMappings":[]}' | shrug jira "workflow scheme drafts" publish-draft-workflow-scheme --id 10001

# Delete the draft
shrug jira "workflow scheme drafts" delete 10001
```

---

## workflow scheme project associations

Query and assign workflow schemes to projects.

**2 operations** (1 CRUD + 1 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get workflow scheme project associations | (query parameters) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `assign-scheme-to-project` | PUT | Assign workflow scheme to project |

### Examples

```bash
# Get workflow scheme associations for a project
shrug jira "workflow scheme project associations" list --projectId 10000

# Assign a workflow scheme to a project
echo '{"workflowSchemeId":"10001","projectId":"10000"}' | shrug jira "workflow scheme project associations" assign-scheme-to-project
```

---

## workflow statuses

Read-only access to Jira workflow statuses.

**2 operations** (2 CRUD)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get all statuses | (no parameters) |
| `get <idOrName>` | Get status | `--idOrName` (path, required) |

### Examples

```bash
# List all statuses
shrug jira "workflow statuses" list

# Get a specific status
shrug jira "workflow statuses" get "In Progress"

# Get status by ID
shrug jira "workflow statuses" get 3
```

---

## workflow status categories

Read-only access to status categories (To Do, In Progress, Done).

**2 operations** (2 CRUD)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get all status categories | (no parameters) |
| `get <idOrKey>` | Get status category | `--idOrKey` (path, required) |

### Examples

```bash
# List all status categories
shrug jira "workflow status categories" list

# Get a specific category
shrug jira "workflow status categories" get 4
```

---

## workflow transition properties

Read and delete properties attached to workflow transitions.

**2 operations** (2 CRUD)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `get <transitionId>` | Get workflow transition properties | `--transitionId` (path, required) |
| `delete <transitionId>` | Delete workflow transition property | `--transitionId` (path, required) |

### Examples

```bash
# Get transition properties
shrug jira "workflow transition properties" get 10001

# Delete a transition property
shrug jira "workflow transition properties" delete 10001 --key myProperty
```

---

## workflow transition rules

Manage workflow transition rule configurations for Connect and Forge apps.

**3 operations** (1 CRUD + 2 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get workflow transition rule configurations | (query parameters) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `update-workflow-transition-rule-configurations` | PUT | Update workflow transition rule configurations |
| `delete-workflow-transition-rule-configurations` | PUT | Delete workflow transition rule configurations |

### Examples

```bash
# Get transition rule configurations
shrug jira "workflow transition rules" list --types postfunction --startAt 0 --maxResults 50

# Update transition rule configurations
echo '{"workflows":[{"workflowId":{"name":"My Workflow"},"rules":[...]}]}' | shrug jira "workflow transition rules" update-workflow-transition-rule-configurations

# Delete transition rule configurations
echo '{"workflows":[{"workflowId":{"name":"My Workflow"},"ruleIds":["rule-123"]}]}' | shrug jira "workflow transition rules" delete-workflow-transition-rule-configurations
```
