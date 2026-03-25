# Confluence: Pages, Blog Posts, and Content

Commands for managing pages, blog posts, folders, whiteboards, content types, custom content, and navigating the content tree (ancestors, children, descendants).

Product alias: `confluence` (or `c`, `conf`)

---

## page

Manage Confluence pages. 8 operations (5 CRUD-mapped, 3 extended).

### CRUD operations

#### list

Get pages.

```
shrug confluence page list [--id ID] [--space-id ID] [--sort SORT] [--status STATUS]
    [--title TITLE] [--body-format FMT] [--subtype TYPE] [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | query | no |
| `--space-id` | query | no |
| `--sort` | query | no |
| `--status` | query | no |
| `--title` | query | no |
| `--body-format` | query | no |
| `--subtype` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

**Examples:**

```bash
# List all pages
shrug confluence page list

# List pages in a specific space
shrug confluence page list --space-id 123456

# Search by title
shrug confluence page list --title "Meeting Notes" --limit 10

# Get pages with body content
shrug confluence page list --body-format storage --limit 5
```

#### create

Create page. Body provided via stdin.

```
shrug confluence page create [--embedded BOOL] [--private BOOL] [--root-level BOOL]
```

| Parameter | Location | Required |
|---|---|---|
| `--embedded` | query | no |
| `--private` | query | no |
| `--root-level` | query | no |

**Example:**

```bash
echo '{"spaceId":"123456","status":"current","title":"New Page","body":{"representation":"storage","value":"<p>Hello world</p>"}}' | shrug confluence page create
```

#### get

Get page by id.

```
shrug confluence page get --id ID [--body-format FMT] [--get-draft BOOL] [--status STATUS]
    [--version N] [--include-labels BOOL] [--include-properties BOOL]
    [--include-operations BOOL] [--include-likes BOOL] [--include-versions BOOL]
    [--include-version BOOL] [--include-favorited-by-current-user-status BOOL]
    [--include-webresources BOOL] [--include-collaborators BOOL]
    [--include-direct-children BOOL]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--body-format` | query | no |
| `--get-draft` | query | no |
| `--status` | query | no |
| `--version` | query | no |
| `--include-labels` | query | no |
| `--include-properties` | query | no |
| `--include-operations` | query | no |
| `--include-likes` | query | no |
| `--include-versions` | query | no |
| `--include-version` | query | no |
| `--include-favorited-by-current-user-status` | query | no |
| `--include-webresources` | query | no |
| `--include-collaborators` | query | no |
| `--include-direct-children` | query | no |

**Examples:**

```bash
# Get page by ID
shrug confluence page get --id 98765

# Get page with body in storage format
shrug confluence page get --id 98765 --body-format storage

# Get page with labels and properties included
shrug confluence page get --id 98765 --include-labels true --include-properties true
```

#### update

Update page. Body provided via stdin.

```
shrug confluence page update --id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |

#### delete

Delete page.

```
shrug confluence page delete --id ID [--purge BOOL] [--draft BOOL]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--purge` | query | no |
| `--draft` | query | no |

**Example:**

```bash
# Move page to trash
shrug confluence page delete --id 98765

# Permanently purge page
shrug confluence page delete --id 98765 --purge true
```

### Extended operations

#### get-label-pages

GET. Get pages for label.

```
shrug confluence page get-label-pages --id ID [--space-id ID] [--body-format FMT]
    [--sort SORT] [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--space-id` | query | no |
| `--body-format` | query | no |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

#### update-page-title

PUT. Update page title.

```
shrug confluence page update-page-title --id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |

#### get-pages-in-space

GET. Get pages in space.

```
shrug confluence page get-pages-in-space --id ID [--depth DEPTH] [--sort SORT]
    [--status STATUS] [--title TITLE] [--body-format FMT] [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--depth` | query | no |
| `--sort` | query | no |
| `--status` | query | no |
| `--title` | query | no |
| `--body-format` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

**Example:**

```bash
shrug confluence page get-pages-in-space --id 123456 --sort "title" --limit 50
```

---

## blog post

Manage Confluence blog posts. 7 operations (5 CRUD-mapped, 2 extended).

### CRUD operations

#### list

Get blog posts.

```
shrug confluence "blog post" list [--id ID] [--space-id ID] [--sort SORT]
    [--status STATUS] [--title TITLE] [--body-format FMT] [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | query | no |
| `--space-id` | query | no |
| `--sort` | query | no |
| `--status` | query | no |
| `--title` | query | no |
| `--body-format` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

**Example:**

```bash
shrug confluence "blog post" list --space-id 123456 --limit 10
```

#### create

Create blog post. Body provided via stdin.

```
shrug confluence "blog post" create [--private BOOL]
```

| Parameter | Location | Required |
|---|---|---|
| `--private` | query | no |

#### get

Get blog post by id.

```
shrug confluence "blog post" get --id ID [--body-format FMT] [--get-draft BOOL]
    [--status STATUS] [--version N] [--include-labels BOOL] [--include-properties BOOL]
    [--include-operations BOOL] [--include-likes BOOL] [--include-versions BOOL]
    [--include-version BOOL] [--include-favorited-by-current-user-status BOOL]
    [--include-webresources BOOL] [--include-collaborators BOOL]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--body-format` | query | no |
| `--get-draft` | query | no |
| `--status` | query | no |
| `--version` | query | no |
| `--include-labels` | query | no |
| `--include-properties` | query | no |
| `--include-operations` | query | no |
| `--include-likes` | query | no |
| `--include-versions` | query | no |
| `--include-version` | query | no |
| `--include-favorited-by-current-user-status` | query | no |
| `--include-webresources` | query | no |
| `--include-collaborators` | query | no |

#### update

Update blog post. Body provided via stdin.

```
shrug confluence "blog post" update --id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |

#### delete

Delete blog post.

```
shrug confluence "blog post" delete --id ID [--purge BOOL] [--draft BOOL]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--purge` | query | no |
| `--draft` | query | no |

### Extended operations

#### get-label-blog-posts

GET. Get blog posts for label.

```
shrug confluence "blog post" get-label-blog-posts --id ID [--space-id ID]
    [--body-format FMT] [--sort SORT] [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--space-id` | query | no |
| `--body-format` | query | no |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

#### get-blog-posts-in-space

GET. Get blog posts in space.

```
shrug confluence "blog post" get-blog-posts-in-space --id ID [--sort SORT]
    [--status STATUS] [--title TITLE] [--body-format FMT] [--cursor CURSOR] [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--sort` | query | no |
| `--status` | query | no |
| `--title` | query | no |
| `--body-format` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |

---

## folder

Manage Confluence folders. 3 operations (3 CRUD-mapped).

### CRUD operations

#### create

Create folder. Body provided via stdin.

```
shrug confluence folder create
```

No query/path parameters.

#### get

Get folder by id.

```
shrug confluence folder get --id ID [--include-collaborators BOOL]
    [--include-direct-children BOOL] [--include-operations BOOL] [--include-properties BOOL]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--include-collaborators` | query | no |
| `--include-direct-children` | query | no |
| `--include-operations` | query | no |
| `--include-properties` | query | no |

**Example:**

```bash
shrug confluence folder get --id 555 --include-direct-children true
```

#### delete

Delete folder.

```
shrug confluence folder delete --id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |

---

## whiteboard

Manage Confluence whiteboards. 3 operations (3 CRUD-mapped).

### CRUD operations

#### create

Create whiteboard. Body provided via stdin.

```
shrug confluence whiteboard create [--private BOOL]
```

| Parameter | Location | Required |
|---|---|---|
| `--private` | query | no |

#### get

Get whiteboard by id.

```
shrug confluence whiteboard get --id ID [--include-collaborators BOOL]
    [--include-direct-children BOOL] [--include-operations BOOL] [--include-properties BOOL]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--include-collaborators` | query | no |
| `--include-direct-children` | query | no |
| `--include-operations` | query | no |
| `--include-properties` | query | no |

#### delete

Delete whiteboard.

```
shrug confluence whiteboard delete --id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |

---

## content

Convert content IDs to content types. 1 operation (1 CRUD-mapped).

### CRUD operations

#### create

Convert content ids to content types. Body provided via stdin.

```
shrug confluence content create
```

No query/path parameters.

---

## custom content

Manage custom content types. 8 operations (5 CRUD-mapped, 3 extended).

### CRUD operations

#### list

Get custom content by type.

```
shrug confluence "custom content" list --type TYPE [--id ID] [--space-id ID]
    [--sort SORT] [--cursor CURSOR] [--limit N] [--body-format FMT]
```

| Parameter | Location | Required |
|---|---|---|
| `--type` | query | yes |
| `--id` | query | no |
| `--space-id` | query | no |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |
| `--body-format` | query | no |

#### create

Create custom content. Body provided via stdin.

```
shrug confluence "custom content" create
```

No query/path parameters.

#### get

Get custom content by id.

```
shrug confluence "custom content" get --id ID [--body-format FMT] [--version N]
    [--include-labels BOOL] [--include-properties BOOL] [--include-operations BOOL]
    [--include-versions BOOL] [--include-version BOOL] [--include-collaborators BOOL]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--body-format` | query | no |
| `--version` | query | no |
| `--include-labels` | query | no |
| `--include-properties` | query | no |
| `--include-operations` | query | no |
| `--include-versions` | query | no |
| `--include-version` | query | no |
| `--include-collaborators` | query | no |

#### update

Update custom content. Body provided via stdin.

```
shrug confluence "custom content" update --id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |

#### delete

Delete custom content.

```
shrug confluence "custom content" delete --id ID [--purge BOOL]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--purge` | query | no |

### Extended operations

#### get-custom-content-by-type-in-blog-post

GET. Get custom content by type in blog post.

```
shrug confluence "custom content" get-custom-content-by-type-in-blog-post --id ID
    --type TYPE [--sort SORT] [--cursor CURSOR] [--limit N] [--body-format FMT]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--type` | query | yes |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |
| `--body-format` | query | no |

#### get-custom-content-by-type-in-page

GET. Get custom content by type in page.

```
shrug confluence "custom content" get-custom-content-by-type-in-page --id ID
    --type TYPE [--sort SORT] [--cursor CURSOR] [--limit N] [--body-format FMT]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--type` | query | yes |
| `--sort` | query | no |
| `--cursor` | query | no |
| `--limit` | query | no |
| `--body-format` | query | no |

#### get-custom-content-by-type-in-space

GET. Get custom content by type in space.

```
shrug confluence "custom content" get-custom-content-by-type-in-space --id ID
    --type TYPE [--cursor CURSOR] [--limit N] [--body-format FMT]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--type` | query | yes |
| `--cursor` | query | no |
| `--limit` | query | no |
| `--body-format` | query | no |

---

## ancestors

Navigate upward through the content tree. 5 operations (1 CRUD-mapped, 4 extended).

### CRUD operations

#### get

Get all ancestors of page.

```
shrug confluence ancestors get --id ID [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--limit` | query | no |

**Example:**

```bash
shrug confluence ancestors get --id 98765
```

### Extended operations

#### get-whiteboard-ancestors

GET. Get all ancestors of whiteboard.

```
shrug confluence ancestors get-whiteboard-ancestors --id ID [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--limit` | query | no |

#### get-database-ancestors

GET. Get all ancestors of database.

```
shrug confluence ancestors get-database-ancestors --id ID [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--limit` | query | no |

#### get-smart-link-ancestors

GET. Get all ancestors of Smart Link in content tree.

```
shrug confluence ancestors get-smart-link-ancestors --id ID [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--limit` | query | no |

#### get-folder-ancestors

GET. Get all ancestors of folder.

```
shrug confluence ancestors get-folder-ancestors --id ID [--limit N]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--limit` | query | no |

---

## children

Get direct children of content nodes. 6 operations (1 CRUD-mapped, 5 extended).

### CRUD operations

#### get

Get direct children of a page.

```
shrug confluence children get --id ID [--cursor CURSOR] [--limit N] [--sort SORT]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--cursor` | query | no |
| `--limit` | query | no |
| `--sort` | query | no |

**Example:**

```bash
shrug confluence children get --id 98765 --sort "title" --limit 25
```

### Extended operations

#### get-whiteboard-direct-children

GET. Get direct children of a whiteboard.

```
shrug confluence children get-whiteboard-direct-children --id ID [--cursor CURSOR]
    [--limit N] [--sort SORT]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--cursor` | query | no |
| `--limit` | query | no |
| `--sort` | query | no |

#### get-database-direct-children

GET. Get direct children of a database.

```
shrug confluence children get-database-direct-children --id ID [--cursor CURSOR]
    [--limit N] [--sort SORT]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--cursor` | query | no |
| `--limit` | query | no |
| `--sort` | query | no |

#### get-smart-link-direct-children

GET. Get direct children of a Smart Link.

```
shrug confluence children get-smart-link-direct-children --id ID [--cursor CURSOR]
    [--limit N] [--sort SORT]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--cursor` | query | no |
| `--limit` | query | no |
| `--sort` | query | no |

#### get-folder-direct-children

GET. Get direct children of a folder.

```
shrug confluence children get-folder-direct-children --id ID [--cursor CURSOR]
    [--limit N] [--sort SORT]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--cursor` | query | no |
| `--limit` | query | no |
| `--sort` | query | no |

#### get-child-custom-content

GET. Get child custom content.

```
shrug confluence children get-child-custom-content --id ID [--cursor CURSOR]
    [--limit N] [--sort SORT]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--cursor` | query | no |
| `--limit` | query | no |
| `--sort` | query | no |

---

## descendants

Get all descendants of content nodes. 5 operations (1 CRUD-mapped, 4 extended).

### CRUD operations

#### get

Get descendants of page.

```
shrug confluence descendants get --id ID [--limit N] [--depth DEPTH] [--cursor CURSOR]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--limit` | query | no |
| `--depth` | query | no |
| `--cursor` | query | no |

**Example:**

```bash
shrug confluence descendants get --id 98765 --depth 3
```

### Extended operations

#### get-whiteboard-descendants

GET. Get descendants of a whiteboard.

```
shrug confluence descendants get-whiteboard-descendants --id ID [--limit N]
    [--depth DEPTH] [--cursor CURSOR]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--limit` | query | no |
| `--depth` | query | no |
| `--cursor` | query | no |

#### get-database-descendants

GET. Get descendants of a database.

```
shrug confluence descendants get-database-descendants --id ID [--limit N]
    [--depth DEPTH] [--cursor CURSOR]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--limit` | query | no |
| `--depth` | query | no |
| `--cursor` | query | no |

#### get-smart-link-descendants

GET. Get descendants of a smart link.

```
shrug confluence descendants get-smart-link-descendants --id ID [--limit N]
    [--depth DEPTH] [--cursor CURSOR]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--limit` | query | no |
| `--depth` | query | no |
| `--cursor` | query | no |

#### get-folder-descendants

GET. Get descendants of folder.

```
shrug confluence descendants get-folder-descendants --id ID [--limit N]
    [--depth DEPTH] [--cursor CURSOR]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--limit` | query | no |
| `--depth` | query | no |
| `--cursor` | query | no |

---

## database

Manage Confluence databases. 3 operations (3 CRUD-mapped).

### CRUD operations

#### create

Create database. Body provided via stdin.

```
shrug confluence database create [--private BOOL]
```

| Parameter | Location | Required |
|---|---|---|
| `--private` | query | no |

#### get

Get database by id.

```
shrug confluence database get --id ID [--include-collaborators BOOL]
    [--include-direct-children BOOL] [--include-operations BOOL] [--include-properties BOOL]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--include-collaborators` | query | no |
| `--include-direct-children` | query | no |
| `--include-operations` | query | no |
| `--include-properties` | query | no |

#### delete

Delete database.

```
shrug confluence database delete --id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |

---

## smart link

Manage Smart Links in the content tree. 3 operations (3 CRUD-mapped).

### CRUD operations

#### create

Create Smart Link in the content tree. Body provided via stdin.

```
shrug confluence "smart link" create
```

No query/path parameters.

#### get

Get Smart Link in the content tree by id.

```
shrug confluence "smart link" get --id ID [--include-collaborators BOOL]
    [--include-direct-children BOOL] [--include-operations BOOL] [--include-properties BOOL]
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
| `--include-collaborators` | query | no |
| `--include-direct-children` | query | no |
| `--include-operations` | query | no |
| `--include-properties` | query | no |

#### delete

Delete Smart Link in the content tree.

```
shrug confluence "smart link" delete --id ID
```

| Parameter | Location | Required |
|---|---|---|
| `--id` | path | yes |
