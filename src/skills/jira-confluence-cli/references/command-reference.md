# shrug Command Reference

Complete command catalogue for shrug, the Atlassian Cloud CLI.

## Table of Contents

- [Authentication & Profiles](#authentication--profiles)
- [Jira Issue](#jira-issue)
- [Jira Issue Comment](#jira-issue-comment)
- [Jira Issue Worklog](#jira-issue-worklog)
- [Jira Issue Attachment](#jira-issue-attachment)
- [Jira Issue Watcher](#jira-issue-watcher)
- [Jira Issue Link](#jira-issue-link)
- [Jira Issue Remote Link](#jira-issue-remote-link)
- [Jira Issue Property](#jira-issue-property)
- [Jira Project](#jira-project)
- [Jira Project Component](#jira-project-component)
- [Jira Project Version](#jira-project-version)
- [Jira Filter](#jira-filter)
- [Jira Dashboard](#jira-dashboard)
- [Jira Label](#jira-label)
- [Jira Audit](#jira-audit)
- [Jira Search](#jira-search)
- [Jira Software Board](#jira-software-board)
- [Jira Software Sprint](#jira-software-sprint)
- [Jira Software Epic](#jira-software-epic)
- [Confluence Space](#confluence-space)
- [Confluence Space Property](#confluence-space-property)
- [Confluence Page](#confluence-page)
- [Confluence Page Comment](#confluence-page-comment)
- [Confluence Page Label](#confluence-page-label)
- [Confluence Page Like](#confluence-page-like)
- [Confluence Page Property](#confluence-page-property)
- [Confluence Page Attachment](#confluence-page-attachment)
- [Confluence Page Version](#confluence-page-version)
- [Confluence Page Restriction](#confluence-page-restriction)
- [Confluence Blog Post](#confluence-blog-post)
- [Confluence Whiteboard](#confluence-whiteboard)
- [Confluence Database](#confluence-database)
- [Confluence Folder](#confluence-folder)
- [Confluence Custom Content](#confluence-custom-content)
- [Confluence Smart Link](#confluence-smart-link)
- [Confluence Task](#confluence-task)
- [Confluence Search](#confluence-search)
- [Template Generation](#template-generation)

---

## Authentication & Profiles

```bash
# Create a profile
shrug profile create <name> --site https://yoursite.atlassian.net --email you@example.com

# Set API token (interactive prompt)
shrug auth set-token --profile <name>

# OAuth 2.0 login (opens browser)
shrug auth login --profile <name>

# Set default profile
shrug profile use <name>

# List profiles
shrug profile list

# View profile details
shrug profile view <name>

# Delete profile
shrug profile delete <name>
```

Environment variable override (takes precedence over profiles):
```bash
export SHRUG_SITE=https://myco.atlassian.net
export SHRUG_EMAIL=me@myco.com
export SHRUG_API_TOKEN=your-token-here
```

---

## Jira Issue

```bash
# List issues
shrug jira issue list --project TEAM
shrug jira issue list --project TEAM --assignee @me --status "In Progress" --type Bug
shrug jira issue list --jql "project = TEAM AND priority = High ORDER BY created DESC"

# Create
shrug jira issue create -s "Fix login bug" --project TEAM --type Bug
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

# Create from JSON
shrug jira issue create -s x --project TEAM --type Task --from-json issue.json

# View
shrug jira issue view TEAM-123

# Edit
shrug jira issue edit TEAM-123 -s "Updated title" --priority Medium
shrug jira issue edit TEAM-123 --add-label urgent --remove-label low-priority

# Delete
shrug jira issue delete TEAM-123 --yes
```

---

## Jira Issue Comment

```bash
# List
shrug jira issue comment list TEAM-123

# Create (markdown auto-converted to ADF)
shrug jira issue comment create TEAM-123 --body "This is **fixed** in v2.1"

# View
shrug jira issue comment view TEAM-123 10001

# Edit
shrug jira issue comment edit TEAM-123 10001 --body "Updated comment"

# Delete
shrug jira issue comment delete TEAM-123 10001 --yes
```

---

## Jira Issue Worklog

```bash
# List
shrug jira issue worklog list TEAM-123

# Create
shrug jira issue worklog create TEAM-123 --time 2h
shrug jira issue worklog create TEAM-123 --time 1h --body "Code review"

# View
shrug jira issue worklog view TEAM-123 20001

# Edit
shrug jira issue worklog edit TEAM-123 20001 --time 3h

# Delete
shrug jira issue worklog delete TEAM-123 20001 --yes
```

---

## Jira Issue Attachment

```bash
# List
shrug jira issue attachment list TEAM-123

# Upload
shrug jira issue attachment create TEAM-123 --file screenshot.png

# View metadata
shrug jira issue attachment view 30001

# Delete
shrug jira issue attachment delete 30001 --yes
```

---

## Jira Issue Watcher

```bash
# List
shrug jira issue watcher list TEAM-123

# Add yourself
shrug jira issue watcher create TEAM-123

# Add specific user
shrug jira issue watcher create TEAM-123 --user 5b10ac8d

# Remove
shrug jira issue watcher delete TEAM-123 --user @me --yes
```

---

## Jira Issue Link

```bash
# List
shrug jira issue link list TEAM-123

# Create
shrug jira issue link create --from TEAM-123 --to TEAM-456 --type Blocks

# View
shrug jira issue link view 40001

# Delete
shrug jira issue link delete 40001 --yes
```

---

## Jira Issue Remote Link

```bash
# List
shrug jira issue remote-link list TEAM-123

# Create
shrug jira issue remote-link create TEAM-123 \
  --url https://github.com/org/repo/pull/42 \
  --title "PR #42" \
  --summary "Fix for login issue"

# View
shrug jira issue remote-link view TEAM-123 50001

# Edit
shrug jira issue remote-link edit TEAM-123 50001 --title "Updated PR"

# Delete
shrug jira issue remote-link delete TEAM-123 50001 --yes
```

---

## Jira Issue Property

```bash
# List
shrug jira issue property list TEAM-123

# Set (creates or updates)
shrug jira issue property edit TEAM-123 my.custom.prop --value '{"enabled":true}'

# View
shrug jira issue property view TEAM-123 my.custom.prop

# Delete
shrug jira issue property delete TEAM-123 my.custom.prop --yes
```

---

## Jira Project

```bash
# List
shrug jira project list
shrug jira project list --type software --query "Team"

# View
shrug jira project view TEAM

# Edit
shrug jira project edit TEAM --name "Renamed Project"

# Delete
shrug jira project delete TEAM --yes
```

---

## Jira Project Component

```bash
# List
shrug jira project component list TEAM

# Create
shrug jira project component create --name Backend --project TEAM --description "API layer"

# View
shrug jira project component view 60001

# Edit
shrug jira project component edit 60001 --name "Backend Services"

# Delete
shrug jira project component delete 60001 --yes
```

---

## Jira Project Version

```bash
# List
shrug jira project version list TEAM

# Create
shrug jira project version create --name v2.0 --project TEAM --release-date 2026-06-30

# View
shrug jira project version view 70001

# Edit
shrug jira project version edit 70001 --name v2.1 --description "Patch release"

# Delete
shrug jira project version delete 70001 --yes
```

---

## Jira Filter

```bash
# List
shrug jira filter list
shrug jira filter list --favourites

# Create
shrug jira filter create --name "My Bugs" --jql "project = TEAM AND type = Bug" --favourite

# View
shrug jira filter view 10100

# Edit
shrug jira filter edit 10100 --name "Updated Filter" --jql "project = TEAM AND type = Bug AND status != Done"

# Delete
shrug jira filter delete 10100 --yes
```

---

## Jira Dashboard

```bash
# List
shrug jira dashboard list

# Create
shrug jira dashboard create --name "Sprint Board" --description "Team sprint tracking"

# View
shrug jira dashboard view 10300

# Edit
shrug jira dashboard edit 10300 --name "Updated Board"

# Delete
shrug jira dashboard delete 10300 --yes
```

---

## Jira Label

```bash
# List all labels (read-only)
shrug jira label list
```

---

## Jira Audit

```bash
# List
shrug jira audit list

# Filter by date range
shrug jira audit list --from 2026-01-01 --to 2026-03-31
```

---

## Jira Search

```bash
# JQL search
shrug jira search list --jql "project = TEAM AND status = Open"

# Shorthand flags
shrug jira search list --project TEAM --assignee @me --type Bug

# With specific fields
shrug jira search list --project TEAM --fields key,summary,status,priority
```

---

## Jira Software Board

```bash
# List
shrug jira-software board list
shrug jira-software board list --project TEAM --type scrum

# Create (requires a filter)
shrug jira-software board create --name "Team Board" --type scrum --filter-id 10100

# View
shrug jira-software board view 42

# View configuration
shrug jira-software board config 42

# Delete
shrug jira-software board delete 42 --yes
```

---

## Jira Software Sprint

```bash
# List
shrug jira-software sprint list --board 42
shrug jira-software sprint list --board 42 --state active

# Create
shrug jira-software sprint create --name "Sprint 1" --board 42 --goal "Ship v2"

# View
shrug jira-software sprint view 99

# Edit
shrug jira-software sprint edit 99 --name "Sprint 1 (extended)" --goal "Updated goal"

# Delete
shrug jira-software sprint delete 99 --yes
```

---

## Jira Software Epic

```bash
# View
shrug jira-software epic view TEAM-50

# List issues in epic
shrug jira-software epic list TEAM-50

# Edit
shrug jira-software epic edit TEAM-50 --name "Renamed Epic" --done
```

---

## Confluence Space

```bash
# List
shrug confluence space list
shrug confluence space list --type global --status current

# Create
shrug confluence space create --key DOCS --name "Documentation"

# View
shrug confluence space view 12345

# Edit (v1 API, uses space key)
shrug confluence space edit DOCS --name "Updated Docs"

# Delete (v1 API, uses space key)
shrug confluence space delete DOCS --yes
```

---

## Confluence Space Property

```bash
# List
shrug confluence space property list 12345

# Create
shrug confluence space property create 12345 --key my.prop --value '{"enabled":true}'

# View
shrug confluence space property view 12345 90001

# Edit
shrug confluence space property edit 12345 90001 --value '{"enabled":false}'

# Delete
shrug confluence space property delete 12345 90001 --yes
```

---

## Confluence Page

```bash
# List
shrug confluence page list
shrug confluence page list --space-id 12345 --status current

# Create
shrug confluence page create --title "Getting Started" --space-id 12345 --body "<p>Welcome</p>"

# Create from JSON
shrug confluence page create --title x --space-id 12345 --from-json page.json

# View
shrug confluence page view 1001

# Edit (auto-increments version)
shrug confluence page edit 1001 --title "Updated Title" --body "<p>New content</p>"

# Delete
shrug confluence page delete 1001 --yes
```

---

## Confluence Page Comment

```bash
# List
shrug confluence page comment list 1001

# Create
shrug confluence page comment create 1001 --body "Looks good"

# View
shrug confluence page comment view 80001

# Edit
shrug confluence page comment edit 80001 --body "Updated comment"

# Delete
shrug confluence page comment delete 80001 --yes
```

---

## Confluence Page Label

```bash
# List
shrug confluence page label list 1001

# Add
shrug confluence page label create 1001 important

# Remove
shrug confluence page label delete 1001 important
```

---

## Confluence Page Like

```bash
# View like count
shrug confluence page like view 1001

# List users who liked
shrug confluence page like list 1001
```

---

## Confluence Page Property

```bash
# List
shrug confluence page property list 1001

# Create
shrug confluence page property create 1001 --key editor --value '{"version":"v2"}'

# View
shrug confluence page property view 1001 pp001

# Edit
shrug confluence page property edit 1001 pp001 --value '{"version":"v3"}'

# Delete
shrug confluence page property delete 1001 pp001 --yes
```

---

## Confluence Page Attachment

```bash
# List
shrug confluence page attachment list 1001

# Upload
shrug confluence page attachment create 1001 --file diagram.png

# View metadata
shrug confluence page attachment view att001

# Replace
shrug confluence page attachment edit 1001 att001 --file updated-diagram.png

# Delete
shrug confluence page attachment delete att001 --yes
```

---

## Confluence Page Version

```bash
# List history
shrug confluence page version list 1001

# View specific version
shrug confluence page version view 1001 2
```

---

## Confluence Page Restriction

```bash
# View
shrug confluence page restriction view 1001

# Set read restriction
shrug confluence page restriction edit 1001 read --user 5b10ac8d

# Remove all restrictions
shrug confluence page restriction delete 1001 --yes
```

---

## Confluence Blog Post

```bash
# List
shrug confluence blogpost list
shrug confluence blogpost list --space-id 12345

# Create
shrug confluence blogpost create --title "Release Notes" --space-id 12345 --body "<p>v2.0 shipped</p>"

# View
shrug confluence blogpost view 2001

# Edit (auto-increments version)
shrug confluence blogpost edit 2001 --title "Updated Release Notes"

# Delete
shrug confluence blogpost delete 2001 --yes
```

Blog posts support the same sub-entities as pages: comment, label, property, attachment, version, like, restriction. Syntax is identical, just replace `page` with `blogpost`.

---

## Confluence Whiteboard

```bash
# Create
shrug confluence whiteboard create --title "Sprint Planning" --space-id 12345

# View
shrug confluence whiteboard view 3001

# Delete
shrug confluence whiteboard delete 3001 --yes
```

---

## Confluence Database

```bash
# Create
shrug confluence database create --title "Bug Tracker" --space-id 12345

# View
shrug confluence database view 4001

# Delete
shrug confluence database delete 4001 --yes
```

---

## Confluence Folder

```bash
# Create
shrug confluence folder create --title "Archive" --space-id 12345

# View
shrug confluence folder view 5001

# Delete
shrug confluence folder delete 5001 --yes
```

---

## Confluence Custom Content

```bash
# List
shrug confluence custom-content list --type "ac:my-app:content"

# Create
shrug confluence custom-content create \
  --type "ac:my-app:content" \
  --title "Widget" \
  --space-id 12345 \
  --body "<p>Content</p>"

# View
shrug confluence custom-content view 6001

# Edit (auto-increments version)
shrug confluence custom-content edit 6001 --title "Updated Widget"

# Delete
shrug confluence custom-content delete 6001 --yes
```

---

## Confluence Smart Link

```bash
# Create (positional URL)
shrug confluence smart-link create https://example.com --space-id 12345

# View
shrug confluence smart-link view 7001

# Delete
shrug confluence smart-link delete 7001 --yes
```

---

## Confluence Task

```bash
# List
shrug confluence task list

# View
shrug confluence task view 8001

# Mark complete
shrug confluence task edit 8001 complete

# Mark incomplete
shrug confluence task edit 8001 incomplete
```

---

## Confluence Search

```bash
# CQL search
shrug confluence search list --cql "type = page AND space = DOCS"

# Shorthand flags
shrug confluence search list --space DOCS --type page --title "Getting Started"
```

---

## Template Generation

Generate JSON body scaffolds for `--from-json` usage.

```bash
# Single template
shrug template jira issue create --output-dir ./templates

# All 13 templates
shrug template all --output-dir ./templates
```

Available templates:
- `shrug template jira issue create/edit`
- `shrug template jsw board create`
- `shrug template jsw sprint create/edit`
- `shrug template conf space create/edit`
- `shrug template conf page create/edit`
- `shrug template conf blogpost create/edit`
- `shrug template conf custom-content create/edit`

Workflow: generate template, edit the JSON file, then pass via `--from-json`.
