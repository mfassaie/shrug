# Jira Project Management

Command reference for project-level entities: avatars, categories, components, versions, properties, email, features, roles, role actors, types, templates, permission schemes, validation, and classification levels.

All commands follow the pattern:

```
shrug jira "<tag>" <operation> [--param value]
```

---

## project avatars

Manage project avatars: list, set, delete, and upload.

**5 operations** (4 CRUD + 1 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get all project avatars | (no parameters) |
| `get <projectIdOrKey>` | Get all project avatars | `--projectIdOrKey` (path, required) |
| `update <projectIdOrKey>` | Set project avatar [body required] | `--projectIdOrKey` (path, required) |
| `delete <projectIdOrKey>` | Delete project avatar | `--projectIdOrKey` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `create-project-avatar` | POST | Load project avatar |

### Examples

```bash
# Get all avatars for a project
shrug jira "project avatars" get PROJ

# Set a project avatar
echo '{"id":"10100"}' | shrug jira "project avatars" update PROJ

# Upload a new project avatar
shrug jira "project avatars" create-project-avatar --projectIdOrKey PROJ

# Delete a project avatar
shrug jira "project avatars" delete PROJ --id 10100
```

---

## project categories

Full CRUD for project categories.

**5 operations** (5 CRUD)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get all project categories | (no parameters) |
| `create` | Create project category [body required] | (body only) |
| `get <id>` | Get project category by ID | `--id` (path, required) |
| `update <id>` | Update project category [body required] | `--id` (path, required) |
| `delete <id>` | Delete project category | `--id` (path, required) |

### Examples

```bash
# List all project categories
shrug jira "project categories" list

# Get a specific category
shrug jira "project categories" get 10001

# Create a category
echo '{"name":"Internal","description":"Internal-only projects"}' | shrug jira "project categories" create

# Update a category
echo '{"name":"Internal Projects","description":"Updated description"}' | shrug jira "project categories" update 10001

# Delete a category
shrug jira "project categories" delete 10001
```

---

## project components

Manage project components: CRUD, paginated listing, and issue counts.

**8 operations** (5 CRUD + 3 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Find components for projects | `--projectIdsOrKeys`, `--startAt`, `--maxResults`, `--orderBy`, `--query` (all query) |
| `create` | Create component [body required] | (body only) |
| `get <id>` | Get component | `--id` (path, required) |
| `update <id>` | Update component [body required] | `--id` (path, required) |
| `delete <id>` | Delete component | `--id` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-component-related-issues` | GET | Get component issues count |
| `get-project-components-paginated` | GET | Get project components paginated |
| `get-project-components` | GET | Get project components |

### Examples

```bash
# Find components across projects
shrug jira "project components" list --projectIdsOrKeys PROJ --query "Backend"

# Get a specific component
shrug jira "project components" get 10001

# Get all components for a project
shrug jira "project components" get-project-components --projectIdOrKey PROJ

# Get components paginated
shrug jira "project components" get-project-components-paginated --projectIdOrKey PROJ --maxResults 50

# Get issue count for a component
shrug jira "project components" get-component-related-issues --id 10001

# Create a component
echo '{"name":"Backend","project":"PROJ","leadAccountId":"5b10ac8d82e05b22cc7d4ef5"}' | shrug jira "project components" create

# Update a component
echo '{"name":"Backend API","description":"REST API layer"}' | shrug jira "project components" update 10001

# Delete a component
shrug jira "project components" delete 10001
```

---

## project versions

Manage project versions (releases): CRUD, related work, merging, moving, and issue counts.

**14 operations** (4 CRUD + 10 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `create` | Create version [body required] | (body only) |
| `get <id>` | Get version | `--id` (path, required), `--expand` (query) |
| `update <id>` | Update version [body required] | `--id` (path, required) |
| `delete <id>` | Delete related work | `--id` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-project-versions-paginated` | GET | Get project versions paginated |
| `get-project-versions` | GET | Get project versions |
| `merge-versions` | PUT | Merge versions |
| `move-version` | POST | Move version |
| `get-version-related-issues` | GET | Get version's related issues count |
| `get-related-work` | GET | Get related work |
| `create-related-work` | POST | Create related work |
| `update-related-work` | PUT | Update related work |
| `delete-and-replace-version` | POST | Delete and replace version |
| `get-version-unresolved-issues` | GET | Get version's unresolved issues count |

### Examples

```bash
# Get all versions for a project
shrug jira "project versions" get-project-versions --projectIdOrKey PROJ

# Get versions paginated
shrug jira "project versions" get-project-versions-paginated --projectIdOrKey PROJ --maxResults 25

# Get a specific version
shrug jira "project versions" get 10001

# Create a version
echo '{"name":"v2.0","project":"PROJ","releaseDate":"2024-06-01","description":"Major release"}' | shrug jira "project versions" create

# Update a version
echo '{"name":"v2.0","released":true}' | shrug jira "project versions" update 10001

# Get related issue count
shrug jira "project versions" get-version-related-issues --id 10001

# Get unresolved issue count
shrug jira "project versions" get-version-unresolved-issues --id 10001

# Merge two versions
shrug jira "project versions" merge-versions --id 10001 --moveIssuesTo 10002

# Move a version in the order
echo '{"after":"https://your-site.atlassian.net/rest/api/3/version/10002"}' | shrug jira "project versions" move-version --id 10001

# Delete and replace a version
echo '{"moveFixIssuesTo":10002,"moveAffectedIssuesTo":10002}' | shrug jira "project versions" delete-and-replace-version --id 10001
```

---

## project properties

Manage arbitrary key-value properties on projects.

**4 operations** (3 CRUD + 1 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `get <projectIdOrKey>` | Get project property keys | `--projectIdOrKey` (path, required) |
| `update <projectIdOrKey>` | Set project property [body required] | `--projectIdOrKey` (path, required) |
| `delete <projectIdOrKey>` | Delete project property | `--projectIdOrKey` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-project-property` | GET | Get project property |

### Examples

```bash
# Get property keys for a project
shrug jira "project properties" get PROJ

# Get a specific property
shrug jira "project properties" get-project-property --projectIdOrKey PROJ --propertyKey myapp.settings

# Set a property
echo '{"deployTarget":"production"}' | shrug jira "project properties" update PROJ --propertyKey myapp.settings

# Delete a property
shrug jira "project properties" delete PROJ --propertyKey myapp.settings
```

---

## project email

Get and set the sender email address for a project.

**2 operations** (2 CRUD)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `get <projectId>` | Get project's sender email | `--projectId` (path, required) |
| `update <projectId>` | Set project's sender email [body required] | `--projectId` (path, required) |

### Examples

```bash
# Get the sender email for a project
shrug jira "project email" get 10000

# Set the sender email
echo '{"emailAddress":"project@example.com"}' | shrug jira "project email" update 10000
```

---

## project features

Get and toggle project features (e.g. sprints, estimation).

**2 operations** (2 CRUD)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `get <projectIdOrKey>` | Get project features | `--projectIdOrKey` (path, required) |
| `update <projectIdOrKey>` | Set project feature state [body required] | `--projectIdOrKey` (path, required) |

### Examples

```bash
# Get features for a project
shrug jira "project features" get PROJ

# Enable or disable a feature
echo '{"state":"ENABLED"}' | shrug jira "project features" update PROJ --featureKey jsw.agility.board
```

---

## project roles

Manage project roles: CRUD, project-scoped queries, and partial updates.

**9 operations** (5 CRUD + 4 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get all project roles | (no parameters) |
| `create` | Create project role [body required] | (body only) |
| `get <id>` | Get project role by ID | `--id` (path, required) |
| `update <id>` | Fully update project role [body required] | `--id` (path, required) |
| `delete <id>` | Delete project role | `--id` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-project-roles` | GET | Get project roles for project |
| `get-project-role` | GET | Get project role for project |
| `get-project-role-details` | GET | Get project role details |
| `partial-update-project-role` | POST | Partial update project role |

### Examples

```bash
# List all project roles
shrug jira "project roles" list

# Get a specific role
shrug jira "project roles" get 10001

# Get roles for a specific project
shrug jira "project roles" get-project-roles --projectIdOrKey PROJ

# Get role details for a project
shrug jira "project roles" get-project-role --projectIdOrKey PROJ --id 10001

# Create a project role
echo '{"name":"QA Lead","description":"Quality assurance lead"}' | shrug jira "project roles" create

# Partially update a role
echo '{"name":"QA Manager"}' | shrug jira "project roles" partial-update-project-role --id 10001

# Delete a role
shrug jira "project roles" delete 10001
```

---

## project role actors

Manage actors (users and groups) assigned to project roles.

**6 operations** (3 CRUD + 3 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `get <id>` | Get default actors for project role | `--id` (path, required) |
| `update <id>` | Set actors for project role [body required] | `--id` (path, required) |
| `delete <id>` | Delete default actors from project role | `--id` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `add-actor-users` | POST | Add actors to project role |
| `delete-actor` | DELETE | Delete actors from project role |
| `add-project-role-actors-to-role` | POST | Add default actors to project role |

### Examples

```bash
# Get default actors for a role
shrug jira "project role actors" get 10001

# Add users to a project role
echo '{"user":["5b10ac8d82e05b22cc7d4ef5"]}' | shrug jira "project role actors" add-actor-users --projectIdOrKey PROJ --id 10001

# Remove an actor from a project role
shrug jira "project role actors" delete-actor --projectIdOrKey PROJ --id 10001 --user 5b10ac8d82e05b22cc7d4ef5

# Add default actors to a role
echo '{"user":["5b10ac8d82e05b22cc7d4ef5"],"groupId":["group-id"]}' | shrug jira "project role actors" add-project-role-actors-to-role --id 10001
```

---

## project types

Read-only listing of project types and their licensing status.

**4 operations** (2 CRUD + 2 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get all project types | (no parameters) |
| `get <projectTypeKey>` | Get project type by key | `--projectTypeKey` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-all-accessible-project-types` | GET | Get licensed project types |
| `get-accessible-project-type-by-key` | GET | Get accessible project type by key |

### Examples

```bash
# List all project types
shrug jira "project types" list

# Get a project type by key
shrug jira "project types" get software

# Get licensed project types only
shrug jira "project types" get-all-accessible-project-types
```

---

## project templates

Manage custom project templates: get, create, edit, save, and remove.

**5 operations** (2 CRUD + 3 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Gets a custom project template | (query parameters) |
| `create` | Create custom project [body required] | (body only) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `edit-template` | PUT | Edit a custom project template |
| `remove-template` | DELETE | Deletes a custom project template |
| `save-template` | POST | Save a custom project template |

### Examples

```bash
# Get a project template
shrug jira "project templates" list --templateKey my-template

# Save a custom project template
echo '{"name":"Sprint Template","projectTypeKey":"software"}' | shrug jira "project templates" save-template

# Edit a template
echo '{"name":"Updated Template"}' | shrug jira "project templates" edit-template --templateKey my-template

# Remove a template
shrug jira "project templates" remove-template --templateKey my-template

# Create a project from a template
echo '{"name":"New Project","key":"NEWP","templateKey":"my-template"}' | shrug jira "project templates" create
```

---

## project permission schemes

Get and assign permission schemes to projects, plus issue security queries.

**4 operations** (2 CRUD + 2 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `get <projectKeyOrId>` | Get project issue security levels | `--projectKeyOrId` (path, required) |
| `update <projectKeyOrId>` | Assign permission scheme [body required] | `--projectKeyOrId` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-project-issue-security-scheme` | GET | Get project issue security scheme |
| `get-assigned-permission-scheme` | GET | Get assigned permission scheme |

### Examples

```bash
# Get issue security levels for a project
shrug jira "project permission schemes" get PROJ

# Get the assigned permission scheme
shrug jira "project permission schemes" get-assigned-permission-scheme --projectKeyOrId PROJ

# Get the issue security scheme
shrug jira "project permission schemes" get-project-issue-security-scheme --projectKeyOrId PROJ

# Assign a permission scheme to a project
echo '{"id":10001}' | shrug jira "project permission schemes" update PROJ
```

---

## project key and name validation

Validate project keys and names before creation.

**3 operations** (1 CRUD + 2 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Validate project key | (query parameters) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-valid-project-key` | GET | Get valid project key |
| `get-valid-project-name` | GET | Get valid project name |

### Examples

```bash
# Validate a project key
shrug jira "project key and name validation" list --key PROJ

# Get a valid project key (auto-corrects invalid keys)
shrug jira "project key and name validation" get-valid-project-key --key "My Project"

# Get a valid project name
shrug jira "project key and name validation" get-valid-project-name --name "My Project"
```

---

## project classification levels

Manage data classification levels for projects.

**4 operations** (3 CRUD + 1 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `get <projectIdOrKey>` | Get the classification configuration for a project | `--projectIdOrKey` (path, required) |
| `update <projectIdOrKey>` | Update the default data classification level of a project [body required] | `--projectIdOrKey` (path, required) |
| `delete <projectIdOrKey>` | Remove the default data classification level from a project | `--projectIdOrKey` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-default-project-classification` | GET | Get the default data classification level of a project |

### Examples

```bash
# Get classification configuration for a project
shrug jira "project classification levels" get PROJ

# Get the default classification level
shrug jira "project classification levels" get-default-project-classification --projectIdOrKey PROJ

# Set the default classification level
echo '{"id":"10001"}' | shrug jira "project classification levels" update PROJ

# Remove the default classification level
shrug jira "project classification levels" delete PROJ
```
