# Jira Users and Permissions

Command reference for users, user search, user properties, myself, groups, group and user picker, permissions, and permission schemes.

All commands follow the pattern:

```
shrug jira "<tag>" <operation> [--param value]
```

---

## users

Manage Jira users: create, list, bulk get, columns, email lookup, and group membership.

**12 operations** (2 CRUD + 10 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get all users default | `--startAt`, `--maxResults`, `--expand` (all query) |
| `create` | Create user [body required] | (body only) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-user` | GET | Get user |
| `remove-user` | DELETE | Delete user |
| `bulk-get-users` | GET | Bulk get users |
| `bulk-get-users-migration` | GET | Get account IDs for users |
| `get-user-default-columns` | GET | Get user default columns |
| `set-user-columns` | PUT | Set user default columns |
| `reset-user-columns` | DELETE | Reset user default columns |
| `get-user-email` | GET | Get user email |
| `get-user-email-bulk` | GET | Get user email bulk |
| `get-user-groups` | GET | Get user groups |
| `get-all-users` | GET | Get all users |

### Examples

```bash
# List all users (default, paginated)
shrug jira users list --startAt 0 --maxResults 50

# Get a specific user
shrug jira users get-user --accountId 5b10ac8d82e05b22cc7d4ef5

# Bulk get users by account ID
shrug jira users bulk-get-users --accountId 5b10ac8d82e05b22cc7d4ef5,5b10a2844c20165700ede21g

# Create a user
echo '{"emailAddress":"user@example.com","displayName":"New User"}' | shrug jira users create

# Get user groups
shrug jira users get-user-groups --accountId 5b10ac8d82e05b22cc7d4ef5

# Get user email (admin only)
shrug jira users get-user-email --accountId 5b10ac8d82e05b22cc7d4ef5

# Get and set default columns
shrug jira users get-user-default-columns --accountId 5b10ac8d82e05b22cc7d4ef5

# Delete a user
shrug jira users remove-user --accountId 5b10ac8d82e05b22cc7d4ef5
```

---

## user search

Find users by various criteria: picker, assignable, permissions, queries.

**8 operations** (1 CRUD + 7 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Find users for picker | `--query` (query, required), `--maxResults`, `--showAvatar`, `--exclude`, `--excludeAccountIds`, `--avatarSize`, `--excludeConnectUsers` (all query) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `find-bulk-assignable-users` | GET | Find users assignable to projects |
| `find-assignable-users` | GET | Find users assignable to issues |
| `find-users-with-all-permissions` | GET | Find users with permissions |
| `find-users` | GET | Find users |
| `find-users-by-query` | GET | Find users by query |
| `find-user-keys-by-query` | GET | Find user keys by query |
| `find-users-with-browse-permission` | GET | Find users with browse permission |

### Examples

```bash
# Find users for a picker (autocomplete)
shrug jira "user search" list --query "john"

# Find users assignable to an issue
shrug jira "user search" find-assignable-users --issueKey PROJ-123 --query "jane"

# Find users assignable to a project
shrug jira "user search" find-bulk-assignable-users --projectKeys PROJ --query "dev"

# Find users with specific permissions
shrug jira "user search" find-users-with-all-permissions --permissions BROWSE --projectKey PROJ

# Find users by query
shrug jira "user search" find-users-by-query --query "is assignee of PROJ"

# Find users with browse permission
shrug jira "user search" find-users-with-browse-permission --projectKey PROJ --query "team"
```

---

## user properties

Manage arbitrary key-value properties on user accounts.

**4 operations** (4 CRUD)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get user property keys | (query parameters) |
| `get <propertyKey>` | Get user property | `--propertyKey` (path, required) |
| `update <propertyKey>` | Set user property [body required] | `--propertyKey` (path, required) |
| `delete <propertyKey>` | Delete user property | `--propertyKey` (path, required) |

### Examples

```bash
# List property keys for a user
shrug jira "user properties" list --accountId 5b10ac8d82e05b22cc7d4ef5

# Get a specific property
shrug jira "user properties" get myapp.preferences --accountId 5b10ac8d82e05b22cc7d4ef5

# Set a property
echo '{"theme":"dark","language":"en"}' | shrug jira "user properties" update myapp.preferences --accountId 5b10ac8d82e05b22cc7d4ef5

# Delete a property
shrug jira "user properties" delete myapp.preferences --accountId 5b10ac8d82e05b22cc7d4ef5
```

---

## myself

Get current user information and manage user preferences and locale.

**5 operations** (1 CRUD + 4 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get current user | (no parameters) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-preference` | GET | Get preference |
| `set-preference` | PUT | Set preference |
| `remove-preference` | DELETE | Delete preference |
| `get-locale` | GET | Get locale |

### Examples

```bash
# Get current user info
shrug jira myself list

# Get a preference
shrug jira myself get-preference --key user.notifications.mimetype

# Set a preference
echo '"text/html"' | shrug jira myself set-preference --key user.notifications.mimetype

# Remove a preference
shrug jira myself remove-preference --key user.notifications.mimetype

# Get the user's locale
shrug jira myself get-locale
```

---

## groups

Manage Jira groups: create, delete, member management, and search.

**7 operations** (2 CRUD + 5 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Bulk get groups | `--startAt`, `--maxResults`, `--groupId`, `--groupName`, `--accessType`, `--applicationKey` (all query) |
| `create` | Create group [body required] | (body only) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `remove-group` | DELETE | Remove group |
| `get-users-from-group` | GET | Get users from group |
| `add-user-to-group` | POST | Add user to group |
| `remove-user-from-group` | DELETE | Remove user from group |
| `find-groups` | GET | Find groups |

### Examples

```bash
# List all groups
shrug jira groups list

# List groups with pagination
shrug jira groups list --startAt 0 --maxResults 25

# Find groups by name
shrug jira groups find-groups --query "developers"

# Create a group
echo '{"name":"frontend-team"}' | shrug jira groups create

# Get users in a group
shrug jira groups get-users-from-group --groupname developers --startAt 0 --maxResults 50

# Add a user to a group
echo '{"accountId":"5b10ac8d82e05b22cc7d4ef5"}' | shrug jira groups add-user-to-group --groupId group-uuid

# Remove a user from a group
shrug jira groups remove-user-from-group --groupId group-uuid --accountId 5b10ac8d82e05b22cc7d4ef5

# Delete a group
shrug jira groups remove-group --groupId group-uuid
```

---

## group and user picker

Combined search for users and groups in a single query.

**1 operation** (1 CRUD)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Find users and groups | (query parameters) |

### Examples

```bash
# Search for users and groups matching a query
shrug jira "group and user picker" list --query "dev" --maxResults 20
```

---

## permissions

Query Jira permissions: all permissions, bulk checks, and permitted projects.

**4 operations** (2 CRUD + 2 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get all permissions | (no parameters) |
| `create` | Get bulk permissions [body required] | (body only) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-my-permissions` | GET | Get my permissions |
| `get-permitted-projects` | POST | Get permitted projects |

### Examples

```bash
# List all available permissions
shrug jira permissions list

# Check my permissions for a project
shrug jira permissions get-my-permissions --projectKey PROJ --permissions BROWSE_PROJECTS,CREATE_ISSUES

# Bulk check permissions
echo '{"projectPermissions":[{"permissions":["BROWSE_PROJECTS"],"projects":[10000,10001]}]}' | shrug jira permissions create

# Get projects I have permission to access
echo '{"permissions":["BROWSE"]}' | shrug jira permissions get-permitted-projects
```

---

## permission schemes

Manage permission schemes: CRUD and permission grants within schemes.

**9 operations** (5 CRUD + 4 raw)

### CRUD operations

| Verb | Description | Parameters |
|------|-------------|------------|
| `list` | Get all permission schemes | (query parameters) |
| `create` | Create permission scheme [body required] | (body only) |
| `get <schemeId>` | Get permission scheme | `--schemeId` (path, required) |
| `update <schemeId>` | Update permission scheme [body required] | `--schemeId` (path, required) |
| `delete <schemeId>` | Delete permission scheme | `--schemeId` (path, required) |

### Raw operations

| Operation | Method | Description |
|-----------|--------|-------------|
| `get-permission-scheme-grants` | GET | Get permission scheme grants |
| `create-permission-grant` | POST | Create permission grant |
| `get-permission-scheme-grant` | GET | Get permission scheme grant |
| `delete-permission-scheme-entity` | DELETE | Delete permission scheme grant |

### Examples

```bash
# List all permission schemes
shrug jira "permission schemes" list

# Get a specific permission scheme
shrug jira "permission schemes" get 10001

# Get grants in a permission scheme
shrug jira "permission schemes" get-permission-scheme-grants --schemeId 10001

# Create a permission scheme
echo '{"name":"Restricted Scheme","description":"Limited access"}' | shrug jira "permission schemes" create

# Add a permission grant to a scheme
echo '{"holder":{"type":"group","parameter":"developers"},"permission":"BROWSE_PROJECTS"}' | shrug jira "permission schemes" create-permission-grant --schemeId 10001

# Get a specific grant
shrug jira "permission schemes" get-permission-scheme-grant --schemeId 10001 --permissionId 10200

# Delete a grant
shrug jira "permission schemes" delete-permission-scheme-entity --schemeId 10001 --permissionId 10200

# Delete a permission scheme
shrug jira "permission schemes" delete 10001
```
