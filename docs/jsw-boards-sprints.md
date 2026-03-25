# Jira Software: Boards, Sprints and Agile

Core agile commands for Jira Software boards, sprints, backlogs, epics, and issues.

Product alias: `jira-software` (or `jsw`)

---

## board

Manage Scrum and Kanban boards. 27 operations (5 CRUD-mapped, 22 extended).

### CRUD operations

#### list

Get all boards.

```
shrug jsw board list [--startAt N] [--maxResults N] [--type TYPE] [--name NAME]
    [--projectKeyOrId KEY] [--accountIdLocation ID] [--projectLocation LOC]
    [--includePrivate BOOL] [--negateLocationFiltering BOOL] [--orderBy FIELD]
    [--expand EXPAND] [--projectTypeLocation TYPE] [--filterId ID]
```

| Parameter | Location | Required |
|---|---|---|
| `--startAt` | query | no |
| `--maxResults` | query | no |
| `--type` | query | no |
| `--name` | query | no |
| `--projectKeyOrId` | query | no |
| `--accountIdLocation` | query | no |
| `--projectLocation` | query | no |
| `--includePrivate` | query | no |
| `--negateLocationFiltering` | query | no |
| `--orderBy` | query | no |
| `--expand` | query | no |
| `--projectTypeLocation` | query | no |
| `--filterId` | query | no |

**Examples:**

```bash
# List all boards
shrug jsw board list

# List scrum boards for a project
shrug jsw board list --type scrum --projectKeyOrId MYPROJ

# List boards with name filter
shrug jsw board list --name "Sprint Board" --maxResults 10
```

#### create

Create board. Body fields: `filterId`, `location`, `name`, `type`.

```
shrug jsw board create
```

No query/path parameters. Pass body as JSON via stdin.

**Example:**

```bash
echo '{"name":"My Board","type":"scrum","filterId":10001}' | shrug jsw board create
```

#### get

Get board by ID.

```
shrug jsw board get --boardId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--boardId` | path | yes |

**Example:**

```bash
shrug jsw board get --boardId 42
```

#### update

Toggle features for a board. Body fields: `boardId`, `enabling`, `feature`.

```
shrug jsw board update --boardId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--boardId` | path | yes |

#### delete

Delete board.

```
shrug jsw board delete --boardId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--boardId` | path | yes |

**Example:**

```bash
shrug jsw board delete --boardId 42
```

### Extended operations

#### get-board-by-filter-id

GET. Get board by filter id.

```
shrug jsw board get-board-by-filter-id --filterId ID [--startAt N] [--maxResults N]
```

| Parameter | Location | Required |
|---|---|---|
| `--filterId` | path | yes |
| `--startAt` | query | no |
| `--maxResults` | query | no |

#### get-issues-for-backlog

GET. Get issues for backlog.

```
shrug jsw board get-issues-for-backlog --boardId ID [--startAt N] [--maxResults N]
    [--jql JQL] [--validateQuery BOOL] [--fields FIELDS] [--expand EXPAND]
```

| Parameter | Location | Required |
|---|---|---|
| `--boardId` | path | yes |
| `--startAt` | query | no |
| `--maxResults` | query | no |
| `--jql` | query | no |
| `--validateQuery` | query | no |
| `--fields` | query | no |
| `--expand` | query | no |

**Example:**

```bash
shrug jsw board get-issues-for-backlog --boardId 42 --maxResults 50
```

#### get-configuration

GET. Get board configuration.

```
shrug jsw board get-configuration --boardId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--boardId` | path | yes |

#### get-epics

GET. Get epics on a board.

```
shrug jsw board get-epics --boardId ID [--startAt N] [--maxResults N] [--done BOOL]
```

| Parameter | Location | Required |
|---|---|---|
| `--boardId` | path | yes |
| `--startAt` | query | no |
| `--maxResults` | query | no |
| `--done` | query | no |

**Example:**

```bash
# Get active (not done) epics on board 42
shrug jsw board get-epics --boardId 42 --done false
```

#### get-issues-without-epic-for-board

GET. Get issues without epic for board.

```
shrug jsw board get-issues-without-epic-for-board --boardId ID [--startAt N]
    [--maxResults N] [--jql JQL] [--validateQuery BOOL] [--fields FIELDS] [--expand EXPAND]
```

| Parameter | Location | Required |
|---|---|---|
| `--boardId` | path | yes |
| `--startAt` | query | no |
| `--maxResults` | query | no |
| `--jql` | query | no |
| `--validateQuery` | query | no |
| `--fields` | query | no |
| `--expand` | query | no |

#### get-board-issues-for-epic

GET. Get board issues for a specific epic.

```
shrug jsw board get-board-issues-for-epic --boardId ID --epicId ID [--startAt N]
    [--maxResults N] [--jql JQL] [--validateQuery BOOL] [--fields FIELDS] [--expand EXPAND]
```

| Parameter | Location | Required |
|---|---|---|
| `--boardId` | path | yes |
| `--epicId` | path | yes |
| `--startAt` | query | no |
| `--maxResults` | query | no |
| `--jql` | query | no |
| `--validateQuery` | query | no |
| `--fields` | query | no |
| `--expand` | query | no |

#### get-features-for-board

GET. Get features for board.

```
shrug jsw board get-features-for-board --boardId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--boardId` | path | yes |

#### get-issues-for-board

GET. Get issues for board.

```
shrug jsw board get-issues-for-board --boardId ID [--startAt N] [--maxResults N]
    [--jql JQL] [--validateQuery BOOL] [--fields FIELDS] [--expand EXPAND]
```

| Parameter | Location | Required |
|---|---|---|
| `--boardId` | path | yes |
| `--startAt` | query | no |
| `--maxResults` | query | no |
| `--jql` | query | no |
| `--validateQuery` | query | no |
| `--fields` | query | no |
| `--expand` | query | no |

**Example:**

```bash
shrug jsw board get-issues-for-board --boardId 42 --jql "status = 'In Progress'"
```

#### move-issues-to-board

POST. Move issues to board.

```
shrug jsw board move-issues-to-board --boardId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--boardId` | path | yes |

#### get-projects

GET. Get projects associated with a board.

```
shrug jsw board get-projects --boardId ID [--startAt N] [--maxResults N]
```

| Parameter | Location | Required |
|---|---|---|
| `--boardId` | path | yes |
| `--startAt` | query | no |
| `--maxResults` | query | no |

#### get-projects-full

GET. Get projects full (with details).

```
shrug jsw board get-projects-full --boardId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--boardId` | path | yes |

#### get-board-property-keys

GET. Get board property keys.

```
shrug jsw board get-board-property-keys --boardId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--boardId` | path | yes |

#### get-board-property

GET. Get board property.

```
shrug jsw board get-board-property --boardId ID --propertyKey KEY
```

| Parameter | Location | Required |
|---|---|---|
| `--boardId` | path | yes |
| `--propertyKey` | path | yes |

#### set-board-property

PUT. Set board property.

```
shrug jsw board set-board-property --boardId ID --propertyKey KEY
```

| Parameter | Location | Required |
|---|---|---|
| `--boardId` | path | yes |
| `--propertyKey` | path | yes |

#### delete-board-property

DELETE. Delete board property.

```
shrug jsw board delete-board-property --boardId ID --propertyKey KEY
```

| Parameter | Location | Required |
|---|---|---|
| `--boardId` | path | yes |
| `--propertyKey` | path | yes |

#### get-all-quick-filters

GET. Get all quick filters.

```
shrug jsw board get-all-quick-filters --boardId ID [--startAt N] [--maxResults N]
```

| Parameter | Location | Required |
|---|---|---|
| `--boardId` | path | yes |
| `--startAt` | query | no |
| `--maxResults` | query | no |

#### get-quick-filter

GET. Get quick filter.

```
shrug jsw board get-quick-filter --boardId ID --quickFilterId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--boardId` | path | yes |
| `--quickFilterId` | path | yes |

#### get-reports-for-board

GET. Get reports for board.

```
shrug jsw board get-reports-for-board --boardId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--boardId` | path | yes |

#### get-all-sprints

GET. Get all sprints for a board.

```
shrug jsw board get-all-sprints --boardId ID [--startAt N] [--maxResults N] [--state STATE]
```

| Parameter | Location | Required |
|---|---|---|
| `--boardId` | path | yes |
| `--startAt` | query | no |
| `--maxResults` | query | no |
| `--state` | query | no |

**Example:**

```bash
# Get active sprints for board 42
shrug jsw board get-all-sprints --boardId 42 --state active
```

#### get-board-issues-for-sprint

GET. Get board issues for sprint.

```
shrug jsw board get-board-issues-for-sprint --boardId ID --sprintId ID [--startAt N]
    [--maxResults N] [--jql JQL] [--validateQuery BOOL] [--fields FIELDS] [--expand EXPAND]
```

| Parameter | Location | Required |
|---|---|---|
| `--boardId` | path | yes |
| `--sprintId` | path | yes |
| `--startAt` | query | no |
| `--maxResults` | query | no |
| `--jql` | query | no |
| `--validateQuery` | query | no |
| `--fields` | query | no |
| `--expand` | query | no |

#### get-all-versions

GET. Get all versions for a board.

```
shrug jsw board get-all-versions --boardId ID [--startAt N] [--maxResults N] [--released BOOL]
```

| Parameter | Location | Required |
|---|---|---|
| `--boardId` | path | yes |
| `--startAt` | query | no |
| `--maxResults` | query | no |
| `--released` | query | no |

---

## sprint

Manage sprints. 12 operations (4 CRUD-mapped, 8 extended).

### CRUD operations

#### create

Create sprint. Body fields: `endDate`, `goal`, `name`, `originBoardId`, `startDate`.

```
shrug jsw sprint create
```

No query/path parameters. Pass body as JSON via stdin.

**Example:**

```bash
echo '{"name":"Sprint 1","originBoardId":42,"startDate":"2026-04-01","endDate":"2026-04-14","goal":"Complete auth module"}' | shrug jsw sprint create
```

#### get

Get sprint.

```
shrug jsw sprint get --sprintId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--sprintId` | path | yes |

**Example:**

```bash
shrug jsw sprint get --sprintId 10
```

#### update

Update sprint. Body fields: `completeDate`, `createdDate`, `endDate`, `goal`, `id`.

```
shrug jsw sprint update --sprintId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--sprintId` | path | yes |

#### delete

Delete sprint.

```
shrug jsw sprint delete --sprintId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--sprintId` | path | yes |

### Extended operations

#### partially-update-sprint

POST. Partially update sprint.

```
shrug jsw sprint partially-update-sprint --sprintId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--sprintId` | path | yes |

#### get-issues-for-sprint

GET. Get issues for sprint.

```
shrug jsw sprint get-issues-for-sprint --sprintId ID [--startAt N] [--maxResults N]
    [--jql JQL] [--validateQuery BOOL] [--fields FIELDS] [--expand EXPAND]
```

| Parameter | Location | Required |
|---|---|---|
| `--sprintId` | path | yes |
| `--startAt` | query | no |
| `--maxResults` | query | no |
| `--jql` | query | no |
| `--validateQuery` | query | no |
| `--fields` | query | no |
| `--expand` | query | no |

**Example:**

```bash
shrug jsw sprint get-issues-for-sprint --sprintId 10 --fields "summary,status"
```

#### move-issues-to-sprint-and-rank

POST. Move issues to sprint and rank.

```
shrug jsw sprint move-issues-to-sprint-and-rank --sprintId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--sprintId` | path | yes |

#### get-properties-keys

GET. Get sprint properties keys.

```
shrug jsw sprint get-properties-keys --sprintId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--sprintId` | path | yes |

#### get-property

GET. Get sprint property.

```
shrug jsw sprint get-property --sprintId ID --propertyKey KEY
```

| Parameter | Location | Required |
|---|---|---|
| `--sprintId` | path | yes |
| `--propertyKey` | path | yes |

#### set-property

PUT. Set sprint property.

```
shrug jsw sprint set-property --sprintId ID --propertyKey KEY
```

| Parameter | Location | Required |
|---|---|---|
| `--sprintId` | path | yes |
| `--propertyKey` | path | yes |

#### delete-property

DELETE. Delete sprint property.

```
shrug jsw sprint delete-property --sprintId ID --propertyKey KEY
```

| Parameter | Location | Required |
|---|---|---|
| `--sprintId` | path | yes |
| `--propertyKey` | path | yes |

#### swap-sprint

POST. Swap sprint.

```
shrug jsw sprint swap-sprint --sprintId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--sprintId` | path | yes |

---

## backlog

Move issues to the backlog. 2 operations (1 CRUD-mapped, 1 extended).

### CRUD operations

#### create

Move issues to backlog. Body field: `issues`.

```
shrug jsw backlog create
```

No query/path parameters. Pass body as JSON via stdin.

**Example:**

```bash
echo '{"issues":["PROJ-1","PROJ-2"]}' | shrug jsw backlog create
```

### Extended operations

#### move-issues-to-backlog-for-board

POST. Move issues to backlog for a specific board.

```
shrug jsw backlog move-issues-to-backlog-for-board --boardId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--boardId` | path | yes |

---

## epic

Manage epics. 7 operations (4 CRUD-mapped, 3 extended).

### CRUD operations

#### list

Get issues without epic.

```
shrug jsw epic list [--startAt N] [--maxResults N] [--jql JQL]
    [--validateQuery BOOL] [--fields FIELDS] [--expand EXPAND]
```

| Parameter | Location | Required |
|---|---|---|
| `--startAt` | query | no |
| `--maxResults` | query | no |
| `--jql` | query | no |
| `--validateQuery` | query | no |
| `--fields` | query | no |
| `--expand` | query | no |

#### create

Remove issues from epic. Body field: `issues`.

```
shrug jsw epic create
```

No query/path parameters. Pass body as JSON via stdin.

#### get

Get epic.

```
shrug jsw epic get --epicIdOrKey KEY
```

| Parameter | Location | Required |
|---|---|---|
| `--epicIdOrKey` | path | yes |

**Example:**

```bash
shrug jsw epic get --epicIdOrKey PROJ-100
```

#### update

Rank epics. Body fields: `rankAfterEpic`, `rankBeforeEpic`, `rankCustomFieldId`.

```
shrug jsw epic update --epicIdOrKey KEY
```

| Parameter | Location | Required |
|---|---|---|
| `--epicIdOrKey` | path | yes |

### Extended operations

#### partially-update-epic

POST. Partially update epic.

```
shrug jsw epic partially-update-epic --epicIdOrKey KEY
```

| Parameter | Location | Required |
|---|---|---|
| `--epicIdOrKey` | path | yes |

#### get-issues-for-epic

GET. Get issues for epic.

```
shrug jsw epic get-issues-for-epic --epicIdOrKey KEY [--startAt N] [--maxResults N]
    [--jql JQL] [--validateQuery BOOL] [--fields FIELDS] [--expand EXPAND]
```

| Parameter | Location | Required |
|---|---|---|
| `--epicIdOrKey` | path | yes |
| `--startAt` | query | no |
| `--maxResults` | query | no |
| `--jql` | query | no |
| `--validateQuery` | query | no |
| `--fields` | query | no |
| `--expand` | query | no |

**Example:**

```bash
shrug jsw epic get-issues-for-epic --epicIdOrKey PROJ-100 --fields "summary,status,assignee"
```

#### move-issues-to-epic

POST. Move issues to epic.

```
shrug jsw epic move-issues-to-epic --epicIdOrKey KEY
```

| Parameter | Location | Required |
|---|---|---|
| `--epicIdOrKey` | path | yes |

---

## issue

Manage issues in a Jira Software context. 4 operations (2 CRUD-mapped, 2 extended).

### CRUD operations

#### get

Get issue.

```
shrug jsw issue get --issueIdOrKey KEY [--fields FIELDS] [--expand EXPAND] [--updateHistory BOOL]
```

| Parameter | Location | Required |
|---|---|---|
| `--issueIdOrKey` | path | yes |
| `--fields` | query | no |
| `--expand` | query | no |
| `--updateHistory` | query | no |

**Example:**

```bash
shrug jsw issue get --issueIdOrKey PROJ-42
shrug jsw issue get --issueIdOrKey PROJ-42 --fields "summary,status,priority"
```

#### update

Estimate issue for board. Body field: `value`.

```
shrug jsw issue update --issueIdOrKey KEY --boardId ID
```

| Parameter | Location | Required |
|---|---|---|
| `--issueIdOrKey` | path | yes |
| `--boardId` | query | no |

### Extended operations

#### rank-issues

PUT. Rank issues.

```
shrug jsw issue rank-issues
```

No parameters. Pass body as JSON via stdin.

#### get-issue-estimation-for-board

GET. Get issue estimation for board.

```
shrug jsw issue get-issue-estimation-for-board --issueIdOrKey KEY [--boardId ID]
```

| Parameter | Location | Required |
|---|---|---|
| `--issueIdOrKey` | path | yes |
| `--boardId` | query | no |

**Example:**

```bash
shrug jsw issue get-issue-estimation-for-board --issueIdOrKey PROJ-42 --boardId 10
```
