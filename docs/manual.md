# shrug Usage Manual

A static CLI for Atlassian Cloud (Jira, Jira Software, Confluence).

## Quick Start

```bash
# 1. Create a profile
shrug profile create mysite --site https://yoursite.atlassian.net --email you@example.com

# 2. Set your API token
shrug auth set-token --profile mysite

# 3. Set as default profile
shrug profile use mysite

# 4. Start using
shrug jira issue list --project TEAM
shrug confluence page list
shrug jira-software board list
```

## Authentication

shrug supports API token and OAuth 2.0 authentication.

### Profile Setup

```bash
# Create a profile
shrug profile create work --site https://myco.atlassian.net --email me@myco.com

# Store API token (interactive prompt)
shrug auth set-token --profile work

# Set as default
shrug profile use work

# List profiles
shrug profile list

# View profile details
shrug profile view work

# Delete profile
shrug profile delete work
```

### Environment Variable Override

Environment variables take precedence over stored profiles.

```bash
export SHRUG_SITE=https://myco.atlassian.net
export SHRUG_EMAIL=me@myco.com
export SHRUG_API_TOKEN=your-token-here
```

### OAuth 2.0

```bash
shrug auth login --profile work
```

Opens a browser for the OAuth flow with PKCE.

## Global Flags

These flags work with any command.

| Flag | Short | Description |
|------|-------|-------------|
| `--output <FORMAT>` | `-o` | Output format: `json`, `table` (default), `csv` |
| `--profile <NAME>` | `-p` | Use a specific profile |
| `--dry-run` | `-n` | Show what would happen without making changes |
| `--limit <N>` | `-L` | Maximum results to fetch |
| `--verbose` | `-v` | Increase verbosity (`-v`, `-vv`, `-vvv` for trace) |
| `--web` | `-w` | Open resource in browser |
| `--quiet` | `-q` | Suppress non-essential output |
| `--color <MODE>` | | `auto` (default), `always`, `never` |

### Examples

```bash
# JSON output
shrug -o json jira issue list --project TEAM

# Limit results
shrug -L 5 jira issue list --project TEAM

# Dry run (shows HTTP request without executing)
shrug -n jira issue create -s "Test" --project TEAM --type Task

# CSV export
shrug -o csv jira issue list --project TEAM > issues.csv

# Specific fields in table output
shrug -o table jira issue list --project TEAM --fields key,summary,status
```

---

## Jira

### Issue

```bash
# List issues in a project
shrug jira issue list --project TEAM

# List with filters
shrug jira issue list --project TEAM --assignee @me --status "In Progress" --type Bug

# List with JQL
shrug jira issue list --jql "project = TEAM AND priority = High ORDER BY created DESC"

# Create an issue
shrug jira issue create -s "Fix login bug" --project TEAM --type Bug

# Create with all options
shrug jira issue create \
  -s "Implement dark mode" \
  --project TEAM \
  --type Story \
  --body "Add dark mode support to the dashboard" \
  --assignee @me \
  --priority High \
  --label ui \
  --label design \
  --component Frontend \
  --due-date 2026-06-30

# Create from JSON file
shrug jira issue create -s x --project TEAM --type Task --from-json issue.json

# View an issue
shrug jira issue view TEAM-123

# Edit an issue
shrug jira issue edit TEAM-123 -s "Updated title" --priority Medium

# Edit with label operations
shrug jira issue edit TEAM-123 --add-label urgent --remove-label low-priority

# Delete an issue
shrug jira issue delete TEAM-123 --yes
```

### Issue Comment

```bash
# List comments
shrug jira issue comment list TEAM-123

# Add a comment (markdown converted to ADF)
shrug jira issue comment create TEAM-123 --body "This is **fixed** in v2.1"

# View a comment
shrug jira issue comment view TEAM-123 10001

# Edit a comment
shrug jira issue comment edit TEAM-123 10001 --body "Updated comment"

# Delete a comment
shrug jira issue comment delete TEAM-123 10001 --yes
```

### Issue Worklog

```bash
# List worklogs
shrug jira issue worklog list TEAM-123

# Log time
shrug jira issue worklog create TEAM-123 --time 2h

# Log time with details
shrug jira issue worklog create TEAM-123 --time 1h --body "Code review"

# View a worklog entry
shrug jira issue worklog view TEAM-123 20001

# Edit a worklog
shrug jira issue worklog edit TEAM-123 20001 --time 3h

# Delete a worklog
shrug jira issue worklog delete TEAM-123 20001 --yes
```

### Issue Attachment

```bash
# List attachments
shrug jira issue attachment list TEAM-123

# Upload a file
shrug jira issue attachment create TEAM-123 --file screenshot.png

# View attachment metadata
shrug jira issue attachment view 30001

# Delete an attachment
shrug jira issue attachment delete 30001 --yes
```

### Issue Watcher

```bash
# List watchers
shrug jira issue watcher list TEAM-123

# Add yourself as watcher
shrug jira issue watcher create TEAM-123

# Add a specific user
shrug jira issue watcher create TEAM-123 --user 5b10ac8d

# Remove a watcher
shrug jira issue watcher delete TEAM-123 --user @me --yes
```

### Issue Link

```bash
# List links on an issue
shrug jira issue link list TEAM-123

# Create a link between issues
shrug jira issue link create --from TEAM-123 --to TEAM-456 --type Blocks

# View a link
shrug jira issue link view 40001

# Delete a link
shrug jira issue link delete 40001 --yes
```

### Issue Remote Link

```bash
# List remote links
shrug jira issue remote-link list TEAM-123

# Create a remote link
shrug jira issue remote-link create TEAM-123 \
  --url https://github.com/org/repo/pull/42 \
  --title "PR #42" \
  --summary "Fix for login issue"

# View a remote link
shrug jira issue remote-link view TEAM-123 50001

# Edit a remote link
shrug jira issue remote-link edit TEAM-123 50001 --title "Updated PR"

# Delete a remote link
shrug jira issue remote-link delete TEAM-123 50001 --yes
```

### Issue Property

```bash
# List properties
shrug jira issue property list TEAM-123

# Set a property (creates or updates)
shrug jira issue property edit TEAM-123 my.custom.prop --value '{"enabled":true}'

# View a property
shrug jira issue property view TEAM-123 my.custom.prop

# Delete a property
shrug jira issue property delete TEAM-123 my.custom.prop --yes
```

### Project

```bash
# List projects
shrug jira project list

# List with filters
shrug jira project list --type software --query "Team"

# View a project
shrug jira project view TEAM

# Edit a project
shrug jira project edit TEAM --name "Renamed Project"

# Delete a project
shrug jira project delete TEAM --yes
```

### Project Component

```bash
# List components
shrug jira project component list TEAM

# Create a component
shrug jira project component create --name Backend --project TEAM --description "API layer"

# View a component
shrug jira project component view 60001

# Edit a component
shrug jira project component edit 60001 --name "Backend Services"

# Delete a component
shrug jira project component delete 60001 --yes
```

### Project Version

```bash
# List versions
shrug jira project version list TEAM

# Create a version
shrug jira project version create --name v2.0 --project TEAM --release-date 2026-06-30

# View a version
shrug jira project version view 70001

# Edit a version
shrug jira project version edit 70001 --name v2.1 --description "Patch release"

# Delete a version
shrug jira project version delete 70001 --yes
```

### Filter

```bash
# List filters
shrug jira filter list

# List favourites only
shrug jira filter list --favourites

# Create a filter
shrug jira filter create --name "My Bugs" --jql "project = TEAM AND type = Bug" --favourite

# View a filter
shrug jira filter view 10100

# Edit a filter
shrug jira filter edit 10100 --name "Updated Filter" --jql "project = TEAM AND type = Bug AND status != Done"

# Delete a filter
shrug jira filter delete 10100 --yes
```

### Dashboard

```bash
# List dashboards
shrug jira dashboard list

# Create a dashboard
shrug jira dashboard create --name "Sprint Board" --description "Team sprint tracking"

# View a dashboard
shrug jira dashboard view 10300

# Edit a dashboard
shrug jira dashboard edit 10300 --name "Updated Board"

# Delete a dashboard
shrug jira dashboard delete 10300 --yes
```

### Label

```bash
# List all labels
shrug jira label list
```

### Audit

```bash
# List audit records
shrug jira audit list

# Filter by date range
shrug jira audit list --from 2026-01-01 --to 2026-03-31
```

### Search

```bash
# Search with JQL
shrug jira search list --jql "project = TEAM AND status = Open"

# Search with shorthand flags
shrug jira search list --project TEAM --assignee @me --type Bug

# Search with specific fields
shrug jira search list --project TEAM --fields key,summary,status,priority
```

---

## Jira Software

### Board

```bash
# List boards
shrug jira-software board list

# List boards for a project
shrug jira-software board list --project TEAM --type scrum

# Create a board (requires a filter)
shrug jira-software board create --name "Team Board" --type scrum --filter-id 10100

# View a board
shrug jira-software board view 42

# View board configuration
shrug jira-software board config 42

# Delete a board
shrug jira-software board delete 42 --yes
```

### Sprint

```bash
# List sprints for a board
shrug jira-software sprint list --board 42

# List active sprints only
shrug jira-software sprint list --board 42 --state active

# Create a sprint
shrug jira-software sprint create --name "Sprint 1" --board 42 --goal "Ship v2"

# View a sprint
shrug jira-software sprint view 99

# Edit a sprint
shrug jira-software sprint edit 99 --name "Sprint 1 (extended)" --goal "Updated goal"

# Delete a sprint
shrug jira-software sprint delete 99 --yes
```

### Epic

```bash
# View an epic
shrug jira-software epic view TEAM-50

# List issues in an epic
shrug jira-software epic list TEAM-50

# Edit an epic
shrug jira-software epic edit TEAM-50 --name "Renamed Epic" --done
```

---

## Confluence

### Space

```bash
# List spaces
shrug confluence space list

# List global spaces only
shrug confluence space list --type global --status current

# Create a space
shrug confluence space create --key DOCS --name "Documentation"

# View a space
shrug confluence space view 12345

# Edit a space (v1 API, uses space key)
shrug confluence space edit DOCS --name "Updated Docs"

# Delete a space (v1 API, uses space key)
shrug confluence space delete DOCS --yes
```

### Space Property

```bash
# List properties
shrug confluence space property list 12345

# Create a property
shrug confluence space property create 12345 --key my.prop --value '{"enabled":true}'

# View a property
shrug confluence space property view 12345 90001

# Edit a property
shrug confluence space property edit 12345 90001 --value '{"enabled":false}'

# Delete a property
shrug confluence space property delete 12345 90001 --yes
```

### Page

```bash
# List pages
shrug confluence page list

# List pages in a space
shrug confluence page list --space-id 12345 --status current

# Create a page
shrug confluence page create --title "Getting Started" --space-id 12345 --body "<p>Welcome</p>"

# Create from JSON
shrug confluence page create --title x --space-id 12345 --from-json page.json

# View a page
shrug confluence page view 1001

# Edit a page (auto-increments version)
shrug confluence page edit 1001 --title "Updated Title" --body "<p>New content</p>"

# Delete a page
shrug confluence page delete 1001 --yes
```

### Page Comment

```bash
# List comments
shrug confluence page comment list 1001

# Add a comment
shrug confluence page comment create 1001 --body "Looks good"

# View a comment
shrug confluence page comment view 80001

# Edit a comment
shrug confluence page comment edit 80001 --body "Updated comment"

# Delete a comment
shrug confluence page comment delete 80001 --yes
```

### Page Label

```bash
# List labels
shrug confluence page label list 1001

# Add a label
shrug confluence page label create 1001 important

# Remove a label
shrug confluence page label delete 1001 important
```

### Page Like

```bash
# View like count
shrug confluence page like view 1001

# List users who liked
shrug confluence page like list 1001
```

### Page Property

```bash
# List properties
shrug confluence page property list 1001

# Create a property
shrug confluence page property create 1001 --key editor --value '{"version":"v2"}'

# View a property
shrug confluence page property view 1001 pp001

# Edit a property
shrug confluence page property edit 1001 pp001 --value '{"version":"v3"}'

# Delete a property
shrug confluence page property delete 1001 pp001 --yes
```

### Page Attachment

```bash
# List attachments
shrug confluence page attachment list 1001

# Upload a file
shrug confluence page attachment create 1001 --file diagram.png

# View attachment metadata
shrug confluence page attachment view att001

# Replace an attachment file
shrug confluence page attachment edit 1001 att001 --file updated-diagram.png

# Delete an attachment
shrug confluence page attachment delete att001 --yes
```

### Page Version

```bash
# List version history
shrug confluence page version list 1001

# View a specific version
shrug confluence page version view 1001 2
```

### Page Restriction

```bash
# View restrictions
shrug confluence page restriction view 1001

# Set read restriction to specific users
shrug confluence page restriction edit 1001 read --user 5b10ac8d

# Remove all restrictions
shrug confluence page restriction delete 1001 --yes
```

### Blog Post

```bash
# List blog posts
shrug confluence blogpost list

# List in a specific space
shrug confluence blogpost list --space-id 12345

# Create a blog post
shrug confluence blogpost create --title "Release Notes" --space-id 12345 --body "<p>v2.0 shipped</p>"

# View a blog post
shrug confluence blogpost view 2001

# Edit a blog post (auto-increments version)
shrug confluence blogpost edit 2001 --title "Updated Release Notes"

# Delete a blog post
shrug confluence blogpost delete 2001 --yes
```

Blog posts support the same sub-entities as pages: comment, label, property, attachment, version, like, restriction. Use `shrug confluence blogpost <sub-entity>` with the same syntax as page sub-entities.

### Whiteboard

```bash
# Create a whiteboard
shrug confluence whiteboard create --title "Sprint Planning" --space-id 12345

# View a whiteboard
shrug confluence whiteboard view 3001

# Delete a whiteboard
shrug confluence whiteboard delete 3001 --yes
```

### Database

```bash
# Create a database
shrug confluence database create --title "Bug Tracker" --space-id 12345

# View a database
shrug confluence database view 4001

# Delete a database
shrug confluence database delete 4001 --yes
```

### Folder

```bash
# Create a folder
shrug confluence folder create --title "Archive" --space-id 12345

# View a folder
shrug confluence folder view 5001

# Delete a folder
shrug confluence folder delete 5001 --yes
```

### Custom Content

```bash
# List custom content
shrug confluence custom-content list --type "ac:my-app:content"

# Create custom content
shrug confluence custom-content create \
  --type "ac:my-app:content" \
  --title "Widget" \
  --space-id 12345 \
  --body "<p>Content</p>"

# View custom content
shrug confluence custom-content view 6001

# Edit custom content (auto-increments version)
shrug confluence custom-content edit 6001 --title "Updated Widget"

# Delete custom content
shrug confluence custom-content delete 6001 --yes
```

### Smart Link

```bash
# Create a smart link (embed)
shrug confluence smart-link create https://example.com --space-id 12345

# View a smart link
shrug confluence smart-link view 7001

# Delete a smart link
shrug confluence smart-link delete 7001 --yes
```

### Task

```bash
# List tasks
shrug confluence task list

# View a task
shrug confluence task view 8001

# Mark task complete
shrug confluence task edit 8001 complete

# Mark task incomplete
shrug confluence task edit 8001 incomplete
```

### Search

```bash
# Search with CQL
shrug confluence search list --cql "type = page AND space = DOCS"

# Search with shorthand flags
shrug confluence search list --space DOCS --type page --title "Getting Started"
```

---

## Template Generation

Generate JSON body scaffolds for commands that support `--from-json`.

```bash
# Generate a single template
shrug template jira issue create --output-dir ./templates

# Generate all 13 templates
shrug template all --output-dir ./templates

# Available templates
shrug template jira issue create --output-dir .       # Jira issue create
shrug template jira issue edit --output-dir .          # Jira issue edit
shrug template jsw board create --output-dir .         # JSW board create
shrug template jsw sprint create --output-dir .        # JSW sprint create
shrug template jsw sprint edit --output-dir .          # JSW sprint edit
shrug template conf space create --output-dir .        # Confluence space create
shrug template conf space edit --output-dir .          # Confluence space edit
shrug template conf page create --output-dir .         # Confluence page create
shrug template conf page edit --output-dir .           # Confluence page edit
shrug template conf blogpost create --output-dir .     # Confluence blogpost create
shrug template conf blogpost edit --output-dir .       # Confluence blogpost edit
shrug template conf custom-content create --output-dir .  # Custom content create
shrug template conf custom-content edit --output-dir .    # Custom content edit
```

### Workflow: Template to Resource

```bash
# 1. Generate template
shrug template jira issue create --output-dir .

# 2. Edit the template (fill in your values)
# Edit jira-issue-create.json

# 3. Create the resource
shrug jira issue create -s x --project TEAM --type Task --from-json jira-issue-create.json
```

---

## Command Aliases

| Full | Alias |
|------|-------|
| `shrug jira` | `shrug j` |
| `shrug jira-software` | `shrug jsw` |
| `shrug confluence` | `shrug c` or `shrug conf` |
| `shrug template jira` | `shrug template j` |
| `shrug template jira-software` | `shrug template jsw` |
| `shrug template confluence` | `shrug template c` or `shrug template conf` |

---

## Install Skill

shrug bundles a Claude Code skill for Atlassian CLI assistance. The skill files are embedded in the binary at compile time and extracted to your Claude environment on demand.

### User scope (all projects)

```bash
# Install to ~/.claude/skills/jira-confluence-cli/
shrug install-skill --scope user
```

This makes the skill available in every Claude Code session.

### Project scope (current project only)

```bash
# Install to .claude/skills/jira-confluence-cli/ in the current directory
shrug install-skill --scope project
```

Requires a `.claude/` directory in the current directory (i.e. a Claude Code project). The skill is only available when working in that project.

Re-running the command updates existing files in place.

---

## Common Workflows

### Create an issue and track it

```bash
# Create
shrug jira issue create -s "Implement feature X" --project TEAM --type Story --assignee @me --priority High

# Add a comment
shrug jira issue comment create TEAM-42 --body "Started work on this"

# Log time
shrug jira issue worklog create TEAM-42 --time 2h --body "Initial implementation"

# Upload a screenshot
shrug jira issue attachment create TEAM-42 --file screenshot.png
```

### Manage a Confluence page

```bash
# Create a page
shrug confluence page create --title "Design Doc" --space-id 12345 --body "# Overview"

# Add a label
shrug confluence page label create 1001 architecture

# Add a comment
shrug confluence page comment create 1001 --body "Reviewed and approved"

# View version history
shrug confluence page version list 1001
```

### Sprint management

```bash
# Create a filter (required for board)
shrug jira filter create --name "Team Filter" --jql "project = TEAM"

# Create a board
shrug jira-software board create --name "Team Board" --type scrum --filter-id 10100

# Create a sprint
shrug jira-software sprint create --name "Sprint 1" --board 42 --goal "Ship MVP"

# List sprints
shrug jira-software sprint list --board 42 --state active

# Edit sprint goal
shrug jira-software sprint edit 99 --goal "Updated goal"
```

### Export data

```bash
# Export issues to CSV
shrug -o csv jira issue list --project TEAM > issues.csv

# Export to JSON
shrug -o json jira issue list --project TEAM > issues.json

# Export with specific fields
shrug -o csv jira issue list --project TEAM --fields key,summary,status,assignee > report.csv
```
