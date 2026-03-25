---
name: jira-confluence-cli
description: "Use the shrug CLI to interact with Atlassian Cloud: Jira issues, projects, filters, dashboards, Jira Software boards/sprints/epics, and Confluence pages, spaces, blog posts, and more. Trigger this skill whenever the user asks you to create, read, update, delete, search, or export anything in Jira or Confluence, manage sprints or boards, look up issues, create pages, or do any Atlassian Cloud operation from the command line. Also trigger when the user mentions shrug, Atlassian, Jira, Confluence, JQL, CQL, sprints, epics, or boards in the context of CLI work."
---

# shrug: Atlassian Cloud CLI

shrug is a CLI installed on this machine that talks to Atlassian Cloud. It covers three products: **Jira** (issues, projects, filters, dashboards, search), **Jira Software** (boards, sprints, epics), and **Confluence** (spaces, pages, blog posts, whiteboards, databases, folders, custom content, smart links, tasks, search).

Every entity supports some subset of LCRUD verbs: `list`, `create`, `view`, `edit`, `delete`. The CLI is consistent across all entities, so once you know the pattern, you can work with any resource.

## Prerequisites

A shrug profile must already be configured with valid credentials. If commands fail with authentication errors, tell the user to run `shrug auth set-token` or check `shrug profile list`. Do not attempt to configure credentials yourself.

## Command structure

```
shrug [GLOBAL FLAGS] <product> <entity> [sub-entity] <verb> [POSITIONAL] [FLAGS]
```

**Products and aliases:**

| Product | Alias |
|---------|-------|
| `jira` | `j` |
| `jira-software` | `jsw` |
| `confluence` | `c`, `conf` |

**Pattern:** `shrug jira issue list`, `shrug confluence page view 1001`, `shrug jsw sprint create --name "Sprint 1" --board 42`.

Many entities have sub-entities. For example, `jira issue comment`, `jira issue worklog`, `confluence page label`, `confluence page attachment`. The sub-entity follows the parent entity and uses the same verb pattern.

## Global flags

Use these with any command.

| Flag | Short | Purpose |
|------|-------|---------|
| `--output <FORMAT>` | `-o` | `json` (machine-readable), `table` (default, human-readable), `csv` (export) |
| `--profile <NAME>` | `-p` | Target a specific Atlassian site profile |
| `--dry-run` | `-n` | Show the HTTP request without executing it. Use this to preview destructive operations |
| `--limit <N>` | `-L` | Cap the number of results returned |
| `--verbose` | `-v` | Increase verbosity. `-vv` for debug, `-vvv` for trace |
| `--web` | `-w` | Open the resource in the browser instead of printing |
| `--quiet` | `-q` | Suppress non-essential output |
| `--fields` | | Select specific fields for table output (e.g. `--fields key,summary,status`) |

**Always use `-o json` when you need to parse output programmatically.** Table output is for humans. JSON output gives you structured data you can pipe through `jq` or parse in scripts.

**Always use `--dry-run` before destructive operations** (create, edit, delete) if you are unsure of the effect. This shows the exact HTTP request that would be sent.

**Always pass `--yes` to delete commands** to skip the confirmation prompt, since you cannot interact with interactive prompts.

## Choosing the right product

- **Jira**: issues, projects, filters, dashboards, labels, audit logs, JQL search
- **Jira Software** (jsw): boards, sprints, epics. These are agile-specific. Use `jira` for the issues themselves, `jsw` for how they are organised into boards and sprints
- **Confluence**: spaces, pages, blog posts, whiteboards, databases, folders, custom content, smart links, tasks, CQL search

## Key entities quick reference

### Jira

| Entity | Verbs | Notes |
|--------|-------|-------|
| `issue` | LCRUD | Core entity. Supports `--jql`, `--project`, `--assignee @me`, `--status`, `--type` for filtering |
| `issue comment` | LCRUD | Markdown body auto-converted to ADF |
| `issue worklog` | LCRUD | Use `--time 2h` for time tracking |
| `issue attachment` | LCRD | `--file <path>` to upload |
| `issue watcher` | LCD | `--user @me` or `--user <accountId>` |
| `issue link` | LCRD | `--from`, `--to`, `--type` |
| `issue remote-link` | LCRUD | `--url`, `--title` |
| `issue property` | LCRD + edit | JSON properties on issues |
| `project` | LCRUD | |
| `project component` | LCRUD | |
| `project version` | LCRUD | |
| `filter` | LCRUD | Saved JQL queries. `--jql`, `--favourite` |
| `dashboard` | LCRUD | |
| `label` | L | Read-only list |
| `audit` | L | `--from`, `--to` for date range |
| `search` | L | JQL search. `--jql` or shorthand flags |

### Jira Software

| Entity | Verbs | Notes |
|--------|-------|-------|
| `board` | LCRD | `--project`, `--type scrum/kanban`. Config via `board config <id>` |
| `sprint` | LCRUD | `--board <id>` required for list/create. `--state active/closed/future` |
| `epic` | view, edit, list | `epic list <key>` lists issues in epic. `--done` to mark complete |

### Confluence

| Entity | Verbs | Notes |
|--------|-------|-------|
| `space` | LCRUD | `--type global`, `--status current` |
| `space property` | LCRUD | |
| `page` | LCRUD | `--space-id` for create/list. Body is storage format HTML |
| `page comment` | LCRUD | |
| `page label` | LCD | Positional label name |
| `page like` | view, list | Read-only |
| `page property` | LCRUD | |
| `page attachment` | LCRUD | `--file` for upload |
| `page version` | view, list | Read-only history |
| `page restriction` | view, edit, delete | Access control |
| `blogpost` | LCRUD | Same sub-entities as page |
| `whiteboard` | CRD | |
| `database` | CRD | |
| `folder` | CRD | |
| `custom-content` | LCRUD | `--type "ac:app:content"` required |
| `smart-link` | CRD | Positional URL |
| `task` | view, list, edit | `edit <id> complete` or `edit <id> incomplete` |
| `search` | L | CQL search. `--cql` or shorthand `--space`, `--type`, `--title` |

## Search

**Jira** uses JQL (Jira Query Language):
```bash
shrug jira search list --jql "project = TEAM AND status = Open"
# Or shorthand:
shrug jira search list --project TEAM --assignee @me --type Bug
```

**Confluence** uses CQL (Confluence Query Language):
```bash
shrug confluence search list --cql "type = page AND space = DOCS"
# Or shorthand:
shrug confluence search list --space DOCS --type page --title "Getting Started"
```

## Creating resources with JSON bodies

For complex creates/edits, generate a template, fill it in, then pass it:

```bash
# 1. Generate template
shrug template jira issue create --output-dir .
# 2. Edit the generated jira-issue-create.json
# 3. Use it
shrug jira issue create -s x --project TEAM --type Task --from-json jira-issue-create.json
```

Template generation supports: `jira issue create/edit`, `jsw board create`, `jsw sprint create/edit`, `conf space create/edit`, `conf page create/edit`, `conf blogpost create/edit`, `conf custom-content create/edit`.

Use `shrug template all --output-dir ./templates` to generate all 13 at once.

## Common workflows

**Create and track an issue:**
```bash
shrug jira issue create -s "Fix login bug" --project TEAM --type Bug --assignee @me
shrug jira issue comment create TEAM-42 --body "Root cause identified"
shrug jira issue worklog create TEAM-42 --time 2h --body "Implemented fix"
```

**Export data:**
```bash
shrug -o csv jira issue list --project TEAM --fields key,summary,status > issues.csv
shrug -o json confluence page list --space-id 12345 > pages.json
```

**Sprint management:**
```bash
shrug jira-software sprint list --board 42 --state active
shrug jira-software sprint create --name "Sprint 5" --board 42 --goal "Ship v2"
```

**Manage a Confluence page:**
```bash
shrug confluence page create --title "Design Doc" --space-id 12345 --body "# Overview"
shrug confluence page label create 1001 architecture
shrug confluence page comment create 1001 --body "Reviewed"
```

## Tips for agents

1. **Parse with JSON.** Always use `-o json` when you need to extract IDs, keys, or other data from command output. Pipe through `jq` for field extraction.

2. **Preview before mutating.** Use `--dry-run` before create/edit/delete if you are unsure about the parameters.

3. **IDs vs keys.** Jira issues use keys like `TEAM-123`. Most other entities use numeric IDs. When you create a resource, capture the ID from the JSON output for subsequent operations.

4. **Sub-entity operations need parent IDs.** To list comments on a page, you need the page ID: `shrug confluence page comment list <page-id>`.

5. **Markdown in Jira.** Comment and issue bodies accept markdown, which shrug auto-converts to Atlassian Document Format (ADF). Use standard markdown syntax.

6. **Confluence body format.** Page and blogpost bodies use HTML storage format. For simple content, use `<p>text</p>`. For richer content, use the `--from-json` approach with a template.

7. **Pagination.** shrug handles pagination automatically. Use `--limit` to cap results.

8. **Discovering structure.** If you don't know the space IDs, project keys, or board IDs, list them first: `shrug confluence space list`, `shrug jira project list`, `shrug jsw board list`.

## Full command reference

For the complete list of every command with all flags and examples, read `references/command-reference.md`. Use it when you need exact flag names or syntax for a specific operation.
