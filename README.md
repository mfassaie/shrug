<p align="center">
  <img src="assets/logo.svg" alt="shrug logo" width="128" height="128">
</p>

<h1 align="center">shrug</h1>

A static CLI for Atlassian Cloud. Manage Jira issues, Jira Software boards, and Confluence pages from your terminal with typed flags and curated commands, without learning the API.

## Features

- **Static command tree**: `shrug <product> <entity> <verb>` with 37 entities and 140+ commands
- **Three Atlassian products** with a single binary: Jira (16 entities), Jira Software (3 entities), Confluence (18 entities)
- **Typed flags** for every command: `--summary`, `--project`, `--type`, `--status`, `--label`, etc.
- **Three input tiers**: typed flags for common fields, `--body`/`--body-file` for descriptions, `--from-json` for full JSON control
- **Three output formats**: JSON, table, CSV with TTY detection
- **Template generation**: `shrug template` generates JSON scaffolds for `--from-json`
- **Claude Code skill**: `shrug install-skill` installs an AI assistant skill for Atlassian workflows
- **Multi-profile authentication** with OS keychain storage, OAuth 2.0 (PKCE), and encrypted file fallback
- **Markdown input**: write issue descriptions in Markdown (auto-converted to ADF or Confluence storage format)
- **JQL shorthand**: `--project`, `--assignee me`, `--status` flags build JQL queries
- **Global flags**: `--dry-run`, `--web`, `--limit`, `--output`, `--fields`
- **Cross-platform**: Windows, macOS, Linux

## Installation

### From GitHub Releases (recommended)

Download the latest binary from [GitHub Releases](https://github.com/mfassaie/shrug/releases).

**Linux/macOS (shell installer):**

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/mfassaie/shrug/releases/latest/download/shrug-installer.sh | sh
```

**Windows (PowerShell installer):**

```powershell
powershell -ExecutionPolicy ByPass -c "irm https://github.com/mfassaie/shrug/releases/latest/download/shrug-installer.ps1 | iex"
```

### From source

```sh
cargo install --git https://github.com/mfassaie/shrug
```

## Quick start

### 1. Create a profile

```sh
shrug profile create work --site mysite.atlassian.net --email me@company.com
shrug auth set-token --profile work
shrug profile use work
```

### 2. Work with Jira issues

```sh
# List issues
shrug jira issue list --project PROJ --status "In Progress"

# Create an issue
shrug jira issue create --summary "Fix login bug" --project PROJ --type Bug

# View an issue
shrug jira issue view PROJ-123

# Edit an issue
shrug jira issue edit PROJ-123 --priority High --label urgent
```

### 3. Work with Confluence pages

```sh
# List pages in a space
shrug confluence page list --space-id 12345

# Create a page with markdown body
shrug confluence page create --space-id 12345 --title "Meeting notes" --body "## Agenda\n- Item one"

# View a page
shrug confluence page view 67890
```

### 4. Work with Jira Software

```sh
# List boards
shrug jira-software board list

# List sprints for a board
shrug jira-software sprint list --board-id 1

# View an epic
shrug jira-software epic view 10001
```

### 5. Use different output formats

```sh
shrug jira issue view PROJ-123 --output json
shrug jira issue list --project PROJ --output csv
shrug jira issue view PROJ-123 --output table --fields key,summary,status
```

### 6. Generate templates for --from-json

```sh
# Generate all templates
shrug template all --output-dir ./templates

# Generate a single template
shrug template jira issue create --output-dir ./templates

# Use a template
shrug jira issue create --from-json templates/jira-issue-create.json
```

### 7. Install the Claude Code skill

```sh
# Install for all projects
shrug install-skill --scope user

# Install for current project only
shrug install-skill --scope project
```

## Command structure

```
shrug <product> <entity> <verb> [flags]
      |          |        |
      |          |        └── list, create, view, edit, delete
      |          └────────── issue, page, board, sprint, space, ...
      └──────────────────── jira, jira-software, confluence
```

Sub-entities nest under their parent: `shrug jira issue comment create PROJ-123 --body "Fixed"`.

## Supported products

| Product | Command | Alias | Entities | Commands |
|---------|---------|-------|----------|----------|
| Jira Cloud | `shrug jira` | `j` | 16 | 63 |
| Jira Software | `shrug jira-software` | `jsw` | 3 | 12 |
| Confluence | `shrug confluence` | `c`, `conf` | 18 | 65 |

## Authentication

shrug supports two authentication methods.

**API token (Basic Auth)** — the simplest option. Generate a token at [id.atlassian.com/manage-profile/security/api-tokens](https://id.atlassian.com/manage-profile/security/api-tokens).

**OAuth 2.0 (3LO with PKCE)** — for automated workflows. Run `shrug auth login` to open a browser flow. Tokens refresh automatically.

Credentials are stored in the OS keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service). An encrypted file fallback is available when the keychain is not accessible.

For CI/CD, set `SHRUG_SITE`, `SHRUG_EMAIL`, and `SHRUG_API_TOKEN` environment variables.

## Configuration

shrug uses layered TOML configuration with this precedence:

1. Command-line flags (highest)
2. Environment variables (`SHRUG_*`)
3. Project config (`.shrug.toml` in current directory or git root)
4. User config (`~/.config/shrug/config.toml`)
5. Built-in defaults (lowest)

## Licence

[MIT](LICENSE)
