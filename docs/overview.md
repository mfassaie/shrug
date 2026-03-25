# shrug CLI Reference

**shrug** is a dynamic CLI for Atlassian Cloud. Commands are generated at runtime from OpenAPI specs, not hardcoded. Version 0.9.0.

---

## Command structure

```
shrug [GLOBAL FLAGS] <product> <tag> <verb> [--param value ...]
```

The four-level hierarchy works like this.

1. **Product** selects the Atlassian API: `jira` (alias `j`), `jira-software` (alias `jsw`), `confluence` (alias `c` or `conf`).
2. **Tag** selects a command group from the OpenAPI spec, e.g. `board`, `page`, `space`. Tags with spaces need quoting: `"blog post"`, `"space roles"`.
3. **Verb** selects the operation. CRUD-mapped verbs (`list`, `create`, `get`, `update`, `delete`) follow a standard pattern. Extended verbs use the full operation name (e.g. `get-all-sprints`).
4. **Parameters** are passed as flags. Path parameters are required. Query parameters are optional. Body data is provided via stdin as JSON.

**Examples:**

```bash
# List all boards (JSW)
shrug jsw board list

# Get a specific page (Confluence)
shrug confluence page get --id 98765

# Create a sprint with body via stdin
echo '{"name":"Sprint 1","originBoardId":42}' | shrug jsw sprint create

# Use JSON output format
shrug -o json confluence space list
```

---

## Global flags

These flags apply to every command.

| Flag | Short | Description | Default |
|---|---|---|---|
| `--output <FORMAT>` | `-o` | Output format: `json`, `table`, `csv` | `table` |
| `--color <MODE>` | | Colour output: `auto`, `always`, `never` | `auto` |
| `--profile <NAME>` | `-p` | Configuration profile to use | default profile |
| `--verbose` | `-v` | Increase verbosity (`-v`, `-vv`, `-vvv` for trace) | off |
| `--dry-run` | `-n` | Show what would happen without making changes | off |
| `--limit <N>` | `-L` | Maximum number of results to fetch (implies pagination) | none |
| `--quiet` | `-q` | Suppress non-essential output | off |
| `--help` | `-h` | Print help | |
| `--version` | `-V` | Print version | |

---

## CRUD verb mapping

shrug maps the five standard CRUD verbs to API operations based on the HTTP method and URL pattern of each endpoint. When a tag has an obvious primary resource, the first matching operation for each verb gets the short alias.

| CRUD verb | Typical HTTP method | Meaning |
|---|---|---|
| `list` | GET (collection) | List or search resources |
| `create` | POST | Create a new resource |
| `get` | GET (single item) | Get a resource by ID |
| `update` | PUT / PATCH | Update an existing resource |
| `delete` | DELETE | Delete a resource |

CRUD verbs accept a positional ID argument when the operation requires a single path parameter. For example, `shrug confluence page get --id 98765` or `shrug jsw board get --boardId 42`.

Operations that don't fit the CRUD pattern are available under their full operation name (the part after the separator line in the tag listing). These are shown with their HTTP method.

---

## Authentication

### auth setup

Interactive setup wizard for first-time configuration. Walks you through creating a profile, setting your site, and storing credentials.

```
shrug auth setup
```

### auth set-token

Store an API token for a profile. Reads the token from stdin for security.

```
shrug auth set-token [--profile NAME]
```

| Option | Description |
|---|---|
| `--profile` | Profile name (uses default if not specified) |

**Example:**

```bash
shrug auth set-token --profile work
```

### auth status

Show credential status for a profile. Reports whether credentials are stored and their type.

```
shrug auth status [--profile NAME]
```

| Option | Description |
|---|---|
| `--profile` | Profile name (uses default if not specified) |

### auth login

Authorize an OAuth 2.0 profile via browser flow. Opens a browser window for the OAuth consent screen, then captures the token via a localhost callback.

```
shrug auth login [--profile NAME]
```

| Option | Description |
|---|---|
| `--profile` | Profile name (uses default if not specified) |

---

## Profile management

Profiles store per-site authentication settings. Each profile has a name, site URL, email, and authentication type.

### profile create

Create a new profile.

```
shrug profile create <NAME> --site <SITE> --email <EMAIL> [--auth-type <TYPE>]
```

| Option | Description | Required |
|---|---|---|
| `<NAME>` | Profile name (lowercase, alphanumeric, hyphens) | yes |
| `--site` | Atlassian site URL (e.g. `mysite.atlassian.net`) | yes |
| `--email` | Email address for authentication | yes |
| `--auth-type` | `basic-auth` (default) or `o-auth2` | no |

**Example:**

```bash
shrug profile create work --site mycompany.atlassian.net --email me@company.com
shrug profile create personal --site personal.atlassian.net --email me@home.com --auth-type o-auth2
```

### profile list

List all profiles.

```
shrug profile list
```

### profile get

Show details of a profile.

```
shrug profile get <NAME>
```

**Example:**

```bash
shrug profile get work
```

### profile update

Update an existing profile. Only the fields you provide will be changed.

```
shrug profile update <NAME> [--site <SITE>] [--email <EMAIL>] [--auth-type <TYPE>]
```

| Option | Description |
|---|---|
| `--site` | New site URL |
| `--email` | New email address |
| `--auth-type` | New authentication type: `basic-auth` or `o-auth2` |

**Example:**

```bash
shrug profile update work --email newemail@company.com
```

### profile delete

Delete a profile.

```
shrug profile delete <NAME>
```

---

## Cache management

shrug caches OpenAPI specs locally to avoid downloading them on every invocation. Specs are fetched from the Atlassian CDN.

### cache list

Show cached API specs with age and status.

```
shrug cache list
```

### cache refresh

Download or refresh API specs from the Atlassian CDN.

```
shrug cache refresh [--product <PRODUCT>]
```

| Option | Description |
|---|---|
| `--product` | Product to refresh: `jira`, `jira-software`, `confluence`. All if not specified |

**Example:**

```bash
# Refresh all specs
shrug cache refresh

# Refresh only Confluence spec
shrug cache refresh --product confluence
```

### cache clear

Delete cached API specs.

```
shrug cache clear [--product <PRODUCT>]
```

| Option | Description |
|---|---|
| `--product` | Product to clear: `jira`, `jira-software`, `confluence`. All if not specified |

---

## Output formats

The `--output` (`-o`) flag controls how results are displayed.

| Format | Description |
|---|---|
| `table` | Human-readable table with aligned columns (default) |
| `json` | Raw JSON response from the API |
| `csv` | Comma-separated values, suitable for piping to other tools |

The `--fields` query parameter (available on many Jira endpoints) lets you select which fields appear in the response.

```bash
# JSON output
shrug -o json confluence page list --limit 5

# CSV output for scripting
shrug -o csv jsw board list | head -5
```

---

## JQL shorthand flags

When using Jira search endpoints that accept a `--jql` parameter, shrug provides shorthand flags that build JQL for you. These flags can be combined with each other and with a raw `--jql` clause.

| Flag | JQL clause generated |
|---|---|
| `--project VALUE` | `project = "VALUE"` |
| `--assignee VALUE` | `assignee = "VALUE"` (or `assignee = currentUser()` if value is `me`) |
| `--status VALUE` | `status = "VALUE"` |
| `--issue-type VALUE` | `issuetype = "VALUE"` |
| `--priority VALUE` | `priority = "VALUE"` |
| `--label VALUE` | `labels = "VALUE"` |

Multiple flags are joined with `AND`. You can still pass a raw `--jql` string, which gets appended.

**Examples:**

```bash
# Using shorthand flags
shrug jira issue search --project MYPROJ --assignee me --status "In Progress"

# Combining shorthand with raw JQL
shrug jira issue search --project MYPROJ --jql "created >= -7d"
```

---

## Configuration

shrug uses a layered TOML configuration system. Settings are applied in order, with later layers overriding earlier ones.

### Layer priority (lowest to highest)

1. Built-in defaults
2. User config file (`config.toml` in the platform config directory)
3. Project config file (`.shrug.toml` in the current directory or any parent up to a `.git` boundary)
4. Environment variables
5. CLI flags

### Config file format

```toml
output_format = "table"     # json, table, csv
color = "auto"              # auto, always, never
default_profile = "work"    # profile name
site = "mysite.atlassian.net"
page_size = 50              # default number of results per API page
cache_ttl_hours = 24        # how long cached specs remain valid
```

All fields are optional. Omitted fields retain their value from the previous layer.

### Config file locations

| Platform | User config path |
|---|---|
| Linux | `~/.config/shrug/config.toml` |
| macOS | `~/Library/Application Support/shrug/config.toml` |
| Windows | `%APPDATA%\shrug\config.toml` |

Project config (`.shrug.toml`) is searched from the current working directory upward, stopping at the first directory containing `.git` or at the filesystem root.

### Environment variables

| Variable | Overrides |
|---|---|
| `SHRUG_OUTPUT` | `output_format` (values: `json`, `table`, `csv`) |
| `SHRUG_COLOR` | `color` (values: `auto`, `always`, `never`) |
| `SHRUG_PROFILE` | `default_profile` |
| `SHRUG_SITE` | `site` |
| `SHRUG_PAGE_SIZE` | `page_size` |

---

## Product documentation

Detailed command reference for each product.

### Jira Software

- [jsw-boards-sprints.md](jsw-boards-sprints.md) - Board, sprint, backlog, epic, and issue commands
- [jsw-devops.md](jsw-devops.md) - Builds, deployments, development information, DevOps components, feature flags, operations, remote links, and security information

### Confluence

- [confluence-pages.md](confluence-pages.md) - Page, blog post, folder, whiteboard, content, custom content, ancestors, children, descendants, database, smart link
- [confluence-spaces.md](confluence-spaces.md) - Space, space permissions, space properties, space roles
- [confluence-content-details.md](confluence-content-details.md) - Attachment, comment, label, like, version, task, content properties, operation, redactions, user, app properties, admin key, classification level, data policies
