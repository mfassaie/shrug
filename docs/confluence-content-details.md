# Confluence: Content Details and Administration

Commands for attachments, comments, labels, likes, versions, tasks, content properties, operations, databases, redactions, users, smart links, app properties, admin keys, classification levels, and data policies.

Product alias: `confluence` (or `c`, `conf`)

---

## attachment

Manage file attachments. 7 operations (3 CRUD-mapped, 4 extended).

### CRUD operations

#### list

Get attachments.

```
shrug confluence attachment list [--sort SORT] [--cursor CURSOR] [--status STATUS]
    [--mediaType TYPE] [--filename NAME] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--sort` | query | no |
| `--cursor` | query | no |
| `--status` | query | no |
| `--mediaType` | query | no |
| `--filename` | query | no |
| `--limit` | query | no |

**Example:**

```bash
shrug confluence attachment list --mediaType "image/png" --limit 20
```

#### get

Get attachment by id.

```
shrug confluence attachment get --id ID [--version N] [--include-labels BOOL]
    [--include-properties BOOL] [--include-operations BOOL] [--include-versions BOOL]
    [--include-version BOOL] [--include-collaborators BOOL]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--version` | query | no |
| `--include-labels` | query | no |
| `--include-properties` | query | no |
| `--include-operations` | query | no |
| `--include-versions` | query | no |
| `--include-version` | query | no |
| `--include-collaborators` | query | no |

#### delete

Delete attachment.

```
shrug confluence attachment delete --id ID [--purge BOOL]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--purge` | query | no |

### Extended operations

#### get-blogpost-attachments

GET. Get attachments for blog post.

```
shrug confluence attachment get-blogpost-attachments --id ID [--sort SORT] [--cursor CURSOR]
    [--status STATUS] [--mediaType TYPE] [--filename NAME] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--status` | query | no |
| `--mediaType` | query | no |
| `--filename` | query | no |
| `--limit` | query | no |

#### get-custom-content-attachments

GET. Get attachments for custom content.

```
shrug confluence attachment get-custom-content-attachments --id ID [--sort SORT]
    [--cursor CURSOR] [--status STATUS] [--mediaType TYPE] [--filename NAME] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--status` | query | no |
| `--mediaType` | query | no |
| `--filename` | query | no |
| `--limit` | query | no |

#### get-label-attachments

GET. Get attachments for label.

```
shrug confluence attachment get-label-attachments --id ID [--sort SORT] [--cursor CURSOR]
    [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

#### get-page-attachments

GET. Get attachments for page.

```
shrug confluence attachment get-page-attachments --id ID [--sort SORT] [--cursor CURSOR]
    [--status STATUS] [--mediaType TYPE] [--filename NAME] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--status` | query | no |
| `--mediaType` | query | no |
| `--filename` | query | no |
| `--limit` | query | no |

**Example:**

```bash
shrug confluence attachment get-page-attachments --id 98765 --filename "report.pdf"
```

---

## comment

Manage footer and inline comments. 18 operations (5 CRUD-mapped, 13 extended).

### CRUD operations

#### list

Get footer comments.

```
shrug confluence comment list [--body-format FMT] [--sort SORT] [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--body-format` | query | no |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

#### create

Create footer comment. Body required, provided via stdin.

```
shrug confluence comment create
```

No query/path parameters.

#### get

Get footer comments for page.

```
shrug confluence comment get --id ID [--body-format FMT] [--status STATUS] [--sort SORT]
    [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--body-format` | query | no |
| `--status` | query | no |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

**Example:**

```bash
shrug confluence comment get --id 98765 --body-format storage
```

#### update

Update footer comment. Body required, provided via stdin.

```
shrug confluence comment update --comment-id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--comment-id` | path | yes |

#### delete

Delete footer comment.

```
shrug confluence comment delete --comment-id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--comment-id` | path | yes |

### Extended operations

#### get-attachment-comments

GET. Get attachment comments.

```
shrug confluence comment get-attachment-comments --id ID [--body-format FMT]
    [--cursor CURSOR] [--limit N] [--sort SORT] [--version N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--body-format` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |
| `--sort` | query | no |
| `--version` | query | no |

#### get-custom-content-comments

GET. Get custom content comments.

```
shrug confluence comment get-custom-content-comments --id ID [--body-format FMT]
    [--cursor CURSOR] [--limit N] [--sort SORT]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--body-format` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |
| `--sort` | query | no |

#### get-page-inline-comments

GET. Get inline comments for page.

```
shrug confluence comment get-page-inline-comments --id ID [--body-format FMT]
    [--status STATUS] [--resolution-status STATUS] [--sort SORT] [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--body-format` | query | no |
| `--status` | query | no |
| `--resolution-status` | query | no |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

#### get-blog-post-footer-comments

GET. Get footer comments for blog post.

```
shrug confluence comment get-blog-post-footer-comments --id ID [--body-format FMT]
    [--status STATUS] [--sort SORT] [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--body-format` | query | no |
| `--status` | query | no |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

#### get-blog-post-inline-comments

GET. Get inline comments for blog post.

```
shrug confluence comment get-blog-post-inline-comments --id ID [--body-format FMT]
    [--status STATUS] [--resolution-status STATUS] [--sort SORT] [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--body-format` | query | no |
| `--status` | query | no |
| `--resolution-status` | query | no |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

#### get-footer-comment-by-id

GET. Get footer comment by id.

```
shrug confluence comment get-footer-comment-by-id --comment-id ID [--body-format FMT]
    [--version N] [--include-properties BOOL] [--include-operations BOOL]
    [--include-likes BOOL] [--include-versions BOOL] [--include-version BOOL]
```

| Parameter | Location | Required |
|---|---|---|
| `--comment-id` | path | yes |
| `--body-format` | query | no |
| `--version` | query | no |
| `--include-properties` | query | no |
| `--include-operations` | query | no |
| `--include-likes` | query | no |
| `--include-versions` | query | no |
| `--include-version` | query | no |

#### get-footer-comment-children

GET. Get children footer comments.

```
shrug confluence comment get-footer-comment-children --id ID [--body-format FMT]
    [--sort SORT] [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--body-format` | query | no |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

#### get-inline-comments

GET. Get inline comments.

```
shrug confluence comment get-inline-comments [--body-format FMT] [--sort SORT]
    [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--body-format` | query | no |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

#### create-inline-comment

POST. Create inline comment. Body provided via stdin.

```
shrug confluence comment create-inline-comment
```

No parameters.

#### get-inline-comment-by-id

GET. Get inline comment by id.

```
shrug confluence comment get-inline-comment-by-id --comment-id ID [--body-format FMT]
    [--version N] [--include-properties BOOL] [--include-operations BOOL]
    [--include-likes BOOL] [--include-versions BOOL] [--include-version BOOL]
```

| Parameter | Location | Required |
|---|---|---|
| `--comment-id` | path | yes |
| `--body-format` | query | no |
| `--version` | query | no |
| `--include-properties` | query | no |
| `--include-operations` | query | no |
| `--include-likes` | query | no |
| `--include-versions` | query | no |
| `--include-version` | query | no |

#### update-inline-comment

PUT. Update inline comment. Body provided via stdin.

```
shrug confluence comment update-inline-comment --comment-id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--comment-id` | path | yes |

#### delete-inline-comment

DELETE. Delete inline comment.

```
shrug confluence comment delete-inline-comment --comment-id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--comment-id` | path | yes |

#### get-inline-comment-children

GET. Get children inline comments.

```
shrug confluence comment get-inline-comment-children --id ID [--body-format FMT]
    [--sort SORT] [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--body-format` | query | no |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

---

## label

View labels across content types. 7 operations (2 CRUD-mapped, 5 extended).

### CRUD operations

#### list

Get labels.

```
shrug confluence label list [--label-id ID] [--prefix PREFIX] [--cursor CURSOR]
    [--sort SORT] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--label-id` | query | no |
| `--prefix` | query | no |
| `--cursor` | query | no |
| `--sort` | query | no |
| `--limit` | query | no |

**Example:**

```bash
shrug confluence label list --prefix "team" --limit 50
```

#### get

Get labels for page.

```
shrug confluence label get --id ID [--prefix PREFIX] [--sort SORT] [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--prefix` | query | no |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

### Extended operations

#### get-attachment-labels

GET. Get labels for attachment.

```
shrug confluence label get-attachment-labels --id ID [--prefix PREFIX] [--sort SORT]
    [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--prefix` | query | no |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

#### get-blog-post-labels

GET. Get labels for blog post.

```
shrug confluence label get-blog-post-labels --id ID [--prefix PREFIX] [--sort SORT]
    [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--prefix` | query | no |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

#### get-custom-content-labels

GET. Get labels for custom content.

```
shrug confluence label get-custom-content-labels --id ID [--prefix PREFIX] [--sort SORT]
    [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--prefix` | query | no |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

#### get-space-labels

GET. Get labels for space.

```
shrug confluence label get-space-labels --id ID [--prefix PREFIX] [--sort SORT]
    [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--prefix` | query | no |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

#### get-space-content-labels

GET. Get labels for space content.

```
shrug confluence label get-space-content-labels --id ID [--prefix PREFIX] [--sort SORT]
    [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--prefix` | query | no |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

---

## like

View like counts and users for content. 8 operations (1 CRUD-mapped, 7 extended).

### CRUD operations

#### get

Get like count for page.

```
shrug confluence like get --id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |

**Example:**

```bash
shrug confluence like get --id 98765
```

### Extended operations

#### get-blog-post-like-count

GET. Get like count for blog post.

```
shrug confluence like get-blog-post-like-count --id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |

#### get-blog-post-like-users

GET. Get account IDs of likes for blog post.

```
shrug confluence like get-blog-post-like-users --id ID [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--cursor` | query | no |
| `--limit` | query | no |

#### get-page-like-users

GET. Get account IDs of likes for page.

```
shrug confluence like get-page-like-users --id ID [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--cursor` | query | no |
| `--limit` | query | no |

#### get-footer-like-count

GET. Get like count for footer comment.

```
shrug confluence like get-footer-like-count --id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |

#### get-footer-like-users

GET. Get account IDs of likes for footer comment.

```
shrug confluence like get-footer-like-users --id ID [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--cursor` | query | no |
| `--limit` | query | no |

#### get-inline-like-count

GET. Get like count for inline comment.

```
shrug confluence like get-inline-like-count --id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |

#### get-inline-like-users

GET. Get account IDs of likes for inline comment.

```
shrug confluence like get-inline-like-users --id ID [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--cursor` | query | no |
| `--limit` | query | no |

---

## version

View version history across content types. 12 operations (1 CRUD-mapped, 11 extended).

### CRUD operations

#### get

Get page versions.

```
shrug confluence version get --id ID [--body-format FMT] [--cursor CURSOR]
    [--limit N] [--sort SORT]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--body-format` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |
| `--sort` | query | no |

**Example:**

```bash
shrug confluence version get --id 98765 --limit 10 --sort "-modified-date"
```

### Extended operations

#### get-attachment-versions

GET. Get attachment versions.

```
shrug confluence version get-attachment-versions --id ID [--cursor CURSOR]
    [--limit N] [--sort SORT]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--cursor` | query | no |
| `--limit` | query | no |
| `--sort` | query | no |

#### get-attachment-version-details

GET. Get version details for attachment version.

```
shrug confluence version get-attachment-version-details --attachment-id ID --version-number N
```

| Parameter | Location | Required |
|---|---|---|
| `--attachment-id` | path | yes |
| `--version-number` | path | yes |

#### get-blog-post-versions

GET. Get blog post versions.

```
shrug confluence version get-blog-post-versions --id ID [--body-format FMT]
    [--cursor CURSOR] [--limit N] [--sort SORT]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--body-format` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |
| `--sort` | query | no |

#### get-blog-post-version-details

GET. Get version details for blog post version.

```
shrug confluence version get-blog-post-version-details --blogpost-id ID --version-number N
```

| Parameter | Location | Required |
|---|---|---|
| `--blogpost-id` | path | yes |
| `--version-number` | path | yes |

#### get-page-version-details

GET. Get version details for page version.

```
shrug confluence version get-page-version-details --page-id ID --version-number N
```

| Parameter | Location | Required |
|---|---|---|
| `--page-id` | path | yes |
| `--version-number` | path | yes |

**Example:**

```bash
shrug confluence version get-page-version-details --page-id 98765 --version-number 3
```

#### get-custom-content-versions

GET. Get custom content versions.

```
shrug confluence version get-custom-content-versions --custom-content-id ID
    [--body-format FMT] [--cursor CURSOR] [--limit N] [--sort SORT]
```

| Parameter | Location | Required |
|---|---|---|
| `--custom-content-id` | path | yes |
| `--body-format` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |
| `--sort` | query | no |

#### get-custom-content-version-details

GET. Get version details for custom content version.

```
shrug confluence version get-custom-content-version-details --custom-content-id ID
    --version-number N
```

| Parameter | Location | Required |
|---|---|---|
| `--custom-content-id` | path | yes |
| `--version-number` | path | yes |

#### get-footer-comment-versions

GET. Get footer comment versions.

```
shrug confluence version get-footer-comment-versions --id ID [--body-format FMT]
    [--cursor CURSOR] [--limit N] [--sort SORT]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--body-format` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |
| `--sort` | query | no |

#### get-footer-comment-version-details

GET. Get version details for footer comment version.

```
shrug confluence version get-footer-comment-version-details --id ID --version-number N
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--version-number` | path | yes |

#### get-inline-comment-versions

GET. Get inline comment versions.

```
shrug confluence version get-inline-comment-versions --id ID [--body-format FMT]
    [--cursor CURSOR] [--limit N] [--sort SORT]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--body-format` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |
| `--sort` | query | no |

#### get-inline-comment-version-details

GET. Get version details for inline comment version.

```
shrug confluence version get-inline-comment-version-details --id ID --version-number N
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--version-number` | path | yes |

---

## task

Manage Confluence tasks. 3 operations (3 CRUD-mapped).

### CRUD operations

#### list

Get tasks.

```
shrug confluence task list [--body-format FMT] [--include-blank-tasks BOOL] [--status STATUS]
    [--task-id ID] [--space-id ID] [--page-id ID] [--blogpost-id ID]
    [--created-by USER] [--assigned-to USER] [--completed-by USER]
    [--created-at-from DATE] [--created-at-to DATE] [--due-at-from DATE] [--due-at-to DATE]
    [--completed-at-from DATE] [--completed-at-to DATE] [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--body-format` | query | no |
| `--include-blank-tasks` | query | no |
| `--status` | query | no |
| `--task-id` | query | no |
| `--space-id` | query | no |
| `--page-id` | query | no |
| `--blogpost-id` | query | no |
| `--created-by` | query | no |
| `--assigned-to` | query | no |
| `--completed-by` | query | no |
| `--created-at-from` | query | no |
| `--created-at-to` | query | no |
| `--due-at-from` | query | no |
| `--due-at-to` | query | no |
| `--completed-at-from` | query | no |
| `--completed-at-to` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

**Examples:**

```bash
# List all open tasks
shrug confluence task list --status incomplete

# List tasks assigned to a user in a space
shrug confluence task list --assigned-to "5b10ac8d" --space-id 123456

# List overdue tasks
shrug confluence task list --due-at-to "2026-03-25" --status incomplete
```

#### get

Get task by id.

```
shrug confluence task get --id ID [--body-format FMT]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--body-format` | query | no |

#### update

Update task.

```
shrug confluence task update --id ID [--body-format FMT]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--body-format` | query | no |

---

## content properties

Manage content properties across all content types. 43 operations (3 CRUD-mapped, 40 extended).

The CRUD-mapped operations target Smart Links. Extended operations cover pages, blog posts, attachments, custom content, whiteboards, databases, folders, and comments.

### CRUD operations

#### get

Get content properties for Smart Link in the content tree.

```
shrug confluence "content properties" get --id ID [--key KEY] [--sort SORT]
    [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--key` | query | no |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

#### update

Update content property for page by id. Body required, provided via stdin.

```
shrug confluence "content properties" update --page-id ID --property-id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--page-id` | path | yes |
| `--property-id` | path | yes |

#### delete

Delete content property for page by id.

```
shrug confluence "content properties" delete --page-id ID --property-id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--page-id` | path | yes |
| `--property-id` | path | yes |

### Extended operations (page)

#### get-page-content-properties

GET. Get content properties for page.

```
shrug confluence "content properties" get-page-content-properties --page-id ID
    [--key KEY] [--sort SORT] [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--page-id` | path | yes |
| `--key` | query | no |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

#### create-page-property

POST. Create content property for page.

```
shrug confluence "content properties" create-page-property --page-id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--page-id` | path | yes |

#### get-page-content-properties-by-id

GET. Get content property for page by id.

```
shrug confluence "content properties" get-page-content-properties-by-id --page-id ID
    --property-id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--page-id` | path | yes |
| `--property-id` | path | yes |

### Extended operations (other content types)

The following extended operations follow the same pattern for each content type. Each type has get, create, get-by-id, update-by-id, and delete-by-id operations.

**Attachment** operations: `get-attachment-content-properties`, `create-attachment-property`, `get-attachment-content-properties-by-id`, `update-attachment-property-by-id`, `delete-attachment-property-by-id`

**Blog post** operations: `get-blogpost-content-properties`, `create-blogpost-property`, `get-blogpost-content-properties-by-id`, `update-blogpost-property-by-id`, `delete-blogpost-property-by-id`

**Custom content** operations: `get-custom-content-content-properties`, `create-custom-content-property`, `get-custom-content-content-properties-by-id`, `update-custom-content-property-by-id`, `delete-custom-content-property-by-id`

**Whiteboard** operations: `get-whiteboard-content-properties`, `create-whiteboard-property`, `get-whiteboard-content-properties-by-id`, `update-whiteboard-property-by-id`, `delete-whiteboard-property-by-id`

**Database** operations: `get-database-content-properties`, `create-database-property`, `get-database-content-properties-by-id`, `update-database-property-by-id`, `delete-database-property-by-id`

**Smart Link** operations: `create-smart-link-property`, `get-smart-link-content-properties-by-id`, `update-smart-link-property-by-id`, `delete-smart-link-property-by-id`

**Folder** operations: `get-folder-content-properties`, `create-folder-property`, `get-folder-content-properties-by-id`, `update-folder-property-by-id`, `delete-folder-property-by-id`

**Comment** operations: `get-comment-content-properties`, `create-comment-property`, `get-comment-content-properties-by-id`, `update-comment-property-by-id`, `delete-comment-property-by-id`

**Example:**

```bash
# Get content properties for a page
shrug confluence "content properties" get-page-content-properties --page-id 98765

# Create a property on a page
echo '{"key":"myProp","value":{"count":42}}' | shrug confluence "content properties" create-page-property --page-id 98765
```

---

## operation

View permitted operations for content. 10 operations (1 CRUD-mapped, 9 extended).

### CRUD operations

#### get

Get permitted operations for page.

```
shrug confluence operation get --id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |

### Extended operations

All extended operations take `--id` (path, required) as their only parameter.

| Operation | Description |
|---|---|
| `get-attachment-operations` | Get permitted operations for attachment |
| `get-blog-post-operations` | Get permitted operations for blog post |
| `get-custom-content-operations` | Get permitted operations for custom content |
| `get-whiteboard-operations` | Get permitted operations for a whiteboard |
| `get-database-operations` | Get permitted operations for a database |
| `get-smart-link-operations` | Get permitted operations for a Smart Link |
| `get-folder-operations` | Get permitted operations for a folder |
| `get-space-operations` | Get permitted operations for space |
| `get-footer-comment-operations` | Get permitted operations for footer comment |
| `get-inline-comment-operations` | Get permitted operations for inline comment |

**Example:**

```bash
shrug confluence operation get --id 98765
shrug confluence operation get-space-operations --id 123456
```

---

## database

See the [confluence-pages.md](confluence-pages.md) file for database CRUD operations (create, get, delete).

---

## redactions

Redact content in pages and blog posts. 2 operations (both extended, no CRUD mapping).

#### post-redact-page

POST. Redact content in a Confluence page.

```
shrug confluence redactions post-redact-page --id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |

#### post-redact-blog

POST. Redact content in a Confluence blog post.

```
shrug confluence redactions post-redact-blog --id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |

---

## user

Manage users and site access. 3 operations (1 CRUD-mapped, 2 extended).

### CRUD operations

#### create

Create bulk user lookup using ids. Body provided via stdin.

```
shrug confluence user create
```

No query/path parameters.

### Extended operations

#### check-access-by-email

POST. Check site access for a list of emails. Body provided via stdin.

```
shrug confluence user check-access-by-email
```

No parameters.

#### invite-by-email

POST. Invite a list of emails to the site. Body provided via stdin.

```
shrug confluence user invite-by-email
```

No parameters.

**Example:**

```bash
echo '{"emails":["user@example.com"]}' | shrug confluence user invite-by-email
```

---

## app properties

Manage Forge app properties. 4 operations (4 CRUD-mapped).

### CRUD operations

#### list

Get Forge app properties.

```
shrug confluence "app properties" list [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--cursor` | query | no |
| `--limit` | query | no |

#### get

Get a Forge app property by key.

```
shrug confluence "app properties" get --propertyKey KEY
```

| Parameter | Location | Required |
|---|---|---|
| `--propertyKey` | path | yes |

#### update

Create or update a Forge app property. Body required, provided via stdin.

```
shrug confluence "app properties" update --propertyKey KEY
```

| Parameter | Location | Required |
|---|---|---|
| `--propertyKey` | path | yes |

#### delete

Deletes a Forge app property.

```
shrug confluence "app properties" delete --propertyKey KEY
```

| Parameter | Location | Required |
|---|---|---|
| `--propertyKey` | path | yes |

---

## admin key

Manage the Confluence admin key. 3 operations (2 CRUD-mapped, 1 extended).

### CRUD operations

#### list

Get admin key status.

```
shrug confluence "admin key" list
```

No parameters.

#### create

Enable admin key.

```
shrug confluence "admin key" create
```

No parameters.

### Extended operations

#### disable-admin-key

DELETE. Disable admin key.

```
shrug confluence "admin key" disable-admin-key
```

No parameters.

---

## classification level

Manage classification levels for content. 16 operations (4 CRUD-mapped, 12 extended).

### CRUD operations

#### list

Get list of classification levels.

```
shrug confluence "classification level" list
```

No parameters.

#### get

Get page classification level.

```
shrug confluence "classification level" get --id ID [--status STATUS]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--status` | query | no |

#### update

Update page classification level.

```
shrug confluence "classification level" update --id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |

#### delete

Delete space default classification level.

```
shrug confluence "classification level" delete --id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |

### Extended operations

Classification levels can be managed for spaces, pages, blog posts, whiteboards, and databases. Each content type has get, put (update), and post (reset) operations.

| Operation | Verb | Description |
|---|---|---|
| `get-space-default-classification-level` | GET | Get space default classification level |
| `put-space-default-classification-level` | PUT | Update space default classification level |
| `post-page-classification-level` | POST | Reset page classification level |
| `get-blog-post-classification-level` | GET | Get blog post classification level |
| `put-blog-post-classification-level` | PUT | Update blog post classification level |
| `post-blog-post-classification-level` | POST | Reset blog post classification level |
| `get-whiteboard-classification-level` | GET | Get whiteboard classification level |
| `put-whiteboard-classification-level` | PUT | Update whiteboard classification level |
| `post-whiteboard-classification-level` | POST | Reset whiteboard classification level |
| `get-database-classification-level` | GET | Get database classification level |
| `put-database-classification-level` | PUT | Update database classification level |
| `post-database-classification-level` | POST | Reset database classification level |

All extended operations take `--id` (path, required) as their parameter. The blog post get operation also accepts `--status` (query, optional).

---

## data policies

View data policy information. 2 operations (1 CRUD-mapped, 1 extended).

### CRUD operations

#### list

Get spaces with data policies.

```
shrug confluence "data policies" list [--ids IDS] [--keys KEYS] [--sort SORT]
    [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--ids` | query | no |
| `--keys` | query | no |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

### Extended operations

#### get-data-policy-metadata

GET. Get data policy metadata for the workspace.

```
shrug confluence "data policies" get-data-policy-metadata
```

No parameters.

---

## smart link

See the [confluence-pages.md](confluence-pages.md) file for Smart Link CRUD operations (create, get, delete).
