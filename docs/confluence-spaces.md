# Confluence: Spaces

Commands for managing Confluence spaces, space permissions, space properties, and space roles.

Product alias: `confluence` (or `c`, `conf`)

---

## space

Manage Confluence spaces. 3 operations (3 CRUD-mapped).

### CRUD operations

#### list

Get spaces.

```
shrug confluence space list [--ids IDS] [--keys KEYS] [--type TYPE] [--status STATUS]
    [--labels LABELS] [--favorited-by USER] [--not-favorited-by USER] [--sort SORT]
    [--description-format FMT] [--include-icon BOOL] [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--ids` | query | no |
| `--keys` | query | no |
| `--type` | query | no |
| `--status` | query | no |
| `--labels` | query | no |
| `--favorited-by` | query | no |
| `--not-favorited-by` | query | no |
| `--sort` | query | no |
| `--description-format` | query | no |
| `--include-icon` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

**Examples:**

```bash
# List all spaces
shrug confluence space list

# List spaces by key
shrug confluence space list --keys "ENG,PROD"

# List global spaces, sorted by name
shrug confluence space list --type global --sort name --limit 20

# Output as JSON
shrug confluence space list -o json
```

#### create

Create space. Body provided via stdin.

```
shrug confluence space create
```

No query/path parameters.

**Example:**

```bash
echo '{"name":"Engineering","key":"ENG","description":{"representation":"plain","value":"Engineering team space"}}' | shrug confluence space create
```

#### get

Get space by id.

```
shrug confluence space get --id ID [--description-format FMT] [--include-icon BOOL]
    [--include-operations BOOL] [--include-properties BOOL] [--include-permissions BOOL]
    [--include-role-assignments BOOL] [--include-labels BOOL]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--description-format` | query | no |
| `--include-icon` | query | no |
| `--include-operations` | query | no |
| `--include-properties` | query | no |
| `--include-permissions` | query | no |
| `--include-role-assignments` | query | no |
| `--include-labels` | query | no |

**Examples:**

```bash
# Get space details
shrug confluence space get --id 123456

# Get space with permissions and labels
shrug confluence space get --id 123456 --include-permissions true --include-labels true
```

---

## space permissions

View space permissions. 2 operations (2 CRUD-mapped).

### CRUD operations

#### list

Get available space permissions.

```
shrug confluence "space permissions" list [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--cursor` | query | no |
| `--limit` | query | no |

**Example:**

```bash
shrug confluence "space permissions" list
```

#### get

Get space permissions assignments for a space.

```
shrug confluence "space permissions" get --id ID [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--cursor` | query | no |
| `--limit` | query | no |

**Example:**

```bash
shrug confluence "space permissions" get --id 123456
```

---

## space properties

Manage space properties. 5 operations (3 CRUD-mapped, 2 extended).

### CRUD operations

#### get

Get space properties in space.

```
shrug confluence "space properties" get --space-id ID [--key KEY] [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--space-id` | path | yes |
| `--key` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

**Example:**

```bash
shrug confluence "space properties" get --space-id 123456
```

#### update

Update space property by id. Body required, provided via stdin.

```
shrug confluence "space properties" update --space-id ID --property-id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--space-id` | path | yes |
| `--property-id` | path | yes |

#### delete

Delete space property by id.

```
shrug confluence "space properties" delete --space-id ID --property-id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--space-id` | path | yes |
| `--property-id` | path | yes |

### Extended operations

#### create-space-property

POST. Create space property in space.

```
shrug confluence "space properties" create-space-property --space-id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--space-id` | path | yes |

#### get-space-property-by-id

GET. Get space property by id.

```
shrug confluence "space properties" get-space-property-by-id --space-id ID --property-id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--space-id` | path | yes |
| `--property-id` | path | yes |

---

## space roles

Manage space roles and role assignments. 8 operations (5 CRUD-mapped, 3 extended).

### CRUD operations

#### list

Get available space roles.

```
shrug confluence "space roles" list [--space-id ID] [--role-type TYPE]
    [--principal-id ID] [--principal-type TYPE] [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--space-id` | query | no |
| `--role-type` | query | no |
| `--principal-id` | query | no |
| `--principal-type` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

**Example:**

```bash
shrug confluence "space roles" list
```

#### create

Create a space role. Body fields: `name*`, `description*`, `spacePermissions*`.

```
shrug confluence "space roles" create
```

No query/path parameters. Body provided via stdin.

**Example:**

```bash
echo '{"name":"Content Editor","description":"Can edit content","spacePermissions":["read","write"]}' | shrug confluence "space roles" create
```

#### get

Get space role by ID.

```
shrug confluence "space roles" get --id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |

#### update

Update a space role. Body fields: `name*`, `description*`, `spacePermissions*`, `anonymousReassignmentRoleId`, `guestReassignmentRoleId`.

```
shrug confluence "space roles" update --id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |

#### delete

Delete a space role.

```
shrug confluence "space roles" delete --id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |

### Extended operations

#### get-space-role-mode

GET. Get space role mode.

```
shrug confluence "space roles" get-space-role-mode
```

No parameters.

#### get-space-role-assignments

GET. Get space role assignments.

```
shrug confluence "space roles" get-space-role-assignments --id ID [--role-id ID]
    [--role-type TYPE] [--principal-id ID] [--principal-type TYPE] [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--role-id` | query | no |
| `--role-type` | query | no |
| `--principal-id` | query | no |
| `--principal-type` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

**Example:**

```bash
shrug confluence "space roles" get-space-role-assignments --id 123456
```

#### set-space-role-assignments

POST. Set space role assignments.

```
shrug confluence "space roles" set-space-role-assignments --id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
