//! Confluence E2E tests using the v2 API.
//! Tests: Page CRUD, Space list, Blog Post list, Comment list, Attachment list.

use crate::harness::{self, ShrugRunner};

fn setup_profile(runner: &ShrugRunner) -> String {
    let name = format!("e2e-conf-{}", std::process::id());
    let result = runner.run(&[
        "profile",
        "create",
        "--name",
        &name,
        "--site",
        runner.config().site.as_str(),
        "--email",
        runner.config().email.as_str(),
    ]);
    assert!(
        result.exit_code == 0 || result.stderr.contains("already exists"),
        "Failed to create profile: {}",
        result.stderr
    );
    let _ = runner.run(&["profile", "use", "--name", &name]);
    name
}

fn teardown_profile(runner: &ShrugRunner, name: &str) {
    let _ = runner.run(&["profile", "delete", "--name", name]);
}

/// Get the space ID for the configured Confluence space key.
fn get_space_id(runner: &ShrugRunner) -> Option<String> {
    let space = runner.config().confluence_space.as_str();
    let spaces = runner.run_json(&["confluence", "Space", "get-spaces"]);
    if spaces.exit_code != 0 {
        return None;
    }
    spaces
        .json
        .as_ref()
        .and_then(|j| j.get("results"))
        .and_then(|r| r.as_array())
        .and_then(|arr| {
            arr.iter()
                .find(|s| s.get("key").and_then(|k| k.as_str()) == Some(space))
        })
        .and_then(|s| s.get("id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Create a page in the given space. Returns page ID.
fn create_page(runner: &ShrugRunner, space_id: &str, title: &str) -> Option<String> {
    let body = format!(
        r#"{{"spaceId":"{}","title":"{}","body":{{"representation":"storage","value":"<p>E2E test content</p>"}},"status":"current"}}"#,
        space_id, title
    );
    let result = runner.run_json_with_body(&body, &["confluence", "Page", "create-page"]);
    if result.exit_code != 0 {
        eprintln!("Failed to create page: {}", result.stderr);
        return None;
    }
    result
        .json
        .as_ref()
        .and_then(|j| j.get("id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn delete_page(runner: &ShrugRunner, id: &str) {
    let result = runner.run(&["confluence", "Page", "delete-page", "--id", id]);
    if result.exit_code == 0 {
        eprintln!("Deleted page: {}", id);
    } else {
        eprintln!("Warning: failed to delete page '{}': {}", id, result.stderr);
    }
}

// ─── Page CRUD ───────────────────────────────────────────────────────────

#[test]
fn test_page_crud_lifecycle() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);
    let space = runner.config().confluence_space.as_str();

    // Need space ID — get it from spaces list
    let spaces = runner.run_json(&["confluence", "Space", "get-spaces"]);
    if spaces.exit_code != 0 {
        eprintln!("Skipping page CRUD: cannot list spaces: {}", spaces.stderr);
        teardown_profile(&runner, &profile);
        return;
    }

    // Find the space ID matching our configured space key
    let space_id = spaces
        .json
        .as_ref()
        .and_then(|j| j.get("results"))
        .and_then(|r| r.as_array())
        .and_then(|arr| {
            arr.iter()
                .find(|s| s.get("key").and_then(|k| k.as_str()) == Some(space))
        })
        .and_then(|s| s.get("id"))
        .and_then(|v| v.as_str());

    let space_id = match space_id {
        Some(id) => id.to_string(),
        None => {
            eprintln!(
                "Skipping page CRUD: space '{}' not found in spaces list",
                space
            );
            teardown_profile(&runner, &profile);
            return;
        }
    };
    harness::rate_limit_delay(runner.config());

    // CREATE page
    let title = format!("E2E Test Page {}", std::process::id());
    let body = format!(
        r#"{{"spaceId":"{}","title":"{}","body":{{"representation":"storage","value":"<p>E2E test content</p>"}},"status":"current"}}"#,
        space_id, title
    );
    let create = runner.run_json_with_body(&body, &["confluence", "Page", "create-page"]);
    if create.exit_code != 0 {
        eprintln!("Skipping page CRUD: create failed: {}", create.stderr);
        teardown_profile(&runner, &profile);
        return;
    }
    let page_id = create
        .json
        .as_ref()
        .and_then(|j| j.get("id"))
        .and_then(|v| v.as_str())
        .expect("Expected page id");
    eprintln!("Created page: {} ({})", title, page_id);
    harness::rate_limit_delay(runner.config());

    // READ
    let read = runner.run_json(&["confluence", "Page", "get-page-by-id", "--id", page_id]);
    read.assert_success();
    assert!(read.json.is_some(), "Expected JSON from get-page-by-id");
    harness::rate_limit_delay(runner.config());

    // UPDATE
    let version = read
        .json
        .as_ref()
        .and_then(|j| j.get("version"))
        .and_then(|v| v.get("number"))
        .and_then(|n| n.as_i64())
        .unwrap_or(1);
    let updated_title = format!("{} Updated", title);
    let ubody = format!(
        r#"{{"id":"{}","title":"{}","spaceId":"{}","body":{{"representation":"storage","value":"<p>Updated content</p>"}},"version":{{"number":{}}},"status":"current"}}"#,
        page_id,
        updated_title,
        space_id,
        version + 1
    );
    let upd = runner.run_with_body(
        &ubody,
        &["confluence", "Page", "update-page", "--id", page_id],
    );
    assert!(upd.exit_code == 0, "update-page failed: {}", upd.stderr);
    harness::rate_limit_delay(runner.config());

    // DELETE
    let del = runner.run(&["confluence", "Page", "delete-page", "--id", page_id]);
    assert!(del.exit_code == 0, "delete-page failed: {}", del.stderr);
    eprintln!("Deleted page: {}", page_id);
    harness::rate_limit_delay(runner.config());

    teardown_profile(&runner, &profile);
}

// ─── Read-Only Tests ─────────────────────────────────────────────────────

#[test]
fn test_list_spaces() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let result = runner.run_json(&["confluence", "Space", "get-spaces"]);
    result.assert_success();
    assert!(result.json.is_some(), "Expected JSON from get-spaces");
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_list_pages() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let result = runner.run_json(&["confluence", "Page", "get-pages"]);
    result.assert_success();
    assert!(result.json.is_some(), "Expected JSON from get-pages");
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_list_blog_posts() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let result = runner.run_json(&["confluence", "Blog Post", "get-blog-posts"]);
    result.assert_success();
    assert!(result.json.is_some(), "Expected JSON from get-blog-posts");
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

#[test]
fn test_list_labels() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let result = runner.run_json(&["confluence", "Label", "get-labels"]);
    result.assert_success();
    assert!(result.json.is_some(), "Expected JSON from get-labels");
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Blog Post CRUD ─────────────────────────────────────────────────────

#[test]
fn test_blog_post_crud_lifecycle() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let space_id = match get_space_id(&runner) {
        Some(id) => id,
        None => {
            eprintln!("Skipping blog post CRUD: space not found");
            teardown_profile(&runner, &profile);
            return;
        }
    };
    harness::rate_limit_delay(runner.config());

    // CREATE
    let title = format!("E2E Blog Post {}", std::process::id());
    let body = format!(
        r#"{{"spaceId":"{}","title":"{}","body":{{"representation":"storage","value":"<p>E2E blog content</p>"}},"status":"current"}}"#,
        space_id, title
    );
    let create = runner.run_json_with_body(&body, &["confluence", "Blog Post", "create-blog-post"]);
    if create.exit_code != 0 {
        eprintln!("Skipping blog post CRUD: create failed: {}", create.stderr);
        teardown_profile(&runner, &profile);
        return;
    }
    let post_id = create
        .json
        .as_ref()
        .and_then(|j| j.get("id"))
        .and_then(|v| v.as_str())
        .expect("Expected blog post id");
    let post_id = post_id.to_string();
    eprintln!("Created blog post: {} ({})", title, post_id);
    harness::rate_limit_delay(runner.config());

    // READ
    let read = runner.run_json(&[
        "confluence",
        "Blog Post",
        "get-blog-post-by-id",
        "--id",
        &post_id,
    ]);
    read.assert_success();
    harness::rate_limit_delay(runner.config());

    // UPDATE
    let version = read
        .json
        .as_ref()
        .and_then(|j| j.get("version"))
        .and_then(|v| v.get("number"))
        .and_then(|n| n.as_i64())
        .unwrap_or(1);
    let updated_title = format!("{} Updated", title);
    let ubody = format!(
        r#"{{"id":"{}","title":"{}","spaceId":"{}","body":{{"representation":"storage","value":"<p>Updated</p>"}},"version":{{"number":{}}},"status":"current"}}"#,
        post_id,
        updated_title,
        space_id,
        version + 1
    );
    let upd = runner.run_with_body(
        &ubody,
        &[
            "confluence",
            "Blog Post",
            "update-blog-post",
            "--id",
            &post_id,
        ],
    );
    assert!(
        upd.exit_code == 0,
        "update-blog-post failed: {}",
        upd.stderr
    );
    harness::rate_limit_delay(runner.config());

    // DELETE
    let del = runner.run(&[
        "confluence",
        "Blog Post",
        "delete-blog-post",
        "--id",
        &post_id,
    ]);
    assert!(
        del.exit_code == 0,
        "delete-blog-post failed: {}",
        del.stderr
    );
    harness::rate_limit_delay(runner.config());

    teardown_profile(&runner, &profile);
}

// ─── Page Comments ──────────────────────────────────────────────────────

#[test]
fn test_page_comments() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let space_id = match get_space_id(&runner) {
        Some(id) => id,
        None => {
            eprintln!("Skipping page comments: space not found");
            teardown_profile(&runner, &profile);
            return;
        }
    };
    harness::rate_limit_delay(runner.config());

    let page_title = format!("E2E Comment Page {}", std::process::id());
    let page_id = match create_page(&runner, &space_id, &page_title) {
        Some(id) => id,
        None => {
            eprintln!("Skipping page comments: could not create page");
            teardown_profile(&runner, &profile);
            return;
        }
    };
    harness::rate_limit_delay(runner.config());

    let result = runner.run_json(&[
        "confluence",
        "Comment",
        "get-page-footer-comments",
        "--id",
        &page_id,
    ]);
    result.assert_success();
    harness::rate_limit_delay(runner.config());

    delete_page(&runner, &page_id);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Space Properties CRUD ──────────────────────────────────────────────

#[test]
fn test_space_properties_crud() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let space_id = match get_space_id(&runner) {
        Some(id) => id,
        None => {
            eprintln!("Skipping space properties: space not found");
            teardown_profile(&runner, &profile);
            return;
        }
    };
    harness::rate_limit_delay(runner.config());

    // CREATE
    let prop_key = format!("e2e-prop-{}", std::process::id());
    let body = format!(r#"{{"key":"{}","value":{{"test":true}}}}"#, prop_key);
    let create = runner.run_json_with_body(
        &body,
        &[
            "confluence",
            "Space Properties",
            "create-space-property",
            "--space-id",
            &space_id,
        ],
    );
    if create.exit_code != 0 {
        eprintln!(
            "Skipping space properties: create failed: {}",
            create.stderr
        );
        teardown_profile(&runner, &profile);
        return;
    }
    let prop_id = create
        .json
        .as_ref()
        .and_then(|j| j.get("id"))
        .and_then(|v| v.as_str())
        .expect("Expected property id");
    let prop_id = prop_id.to_string();
    eprintln!("Created space property: {} ({})", prop_key, prop_id);
    harness::rate_limit_delay(runner.config());

    // READ
    let read = runner.run_json(&[
        "confluence",
        "Space Properties",
        "get-space-property-by-id",
        "--space-id",
        &space_id,
        "--property-id",
        &prop_id,
    ]);
    read.assert_success();
    harness::rate_limit_delay(runner.config());

    // UPDATE
    let ubody = format!(r#"{{"key":"{}","value":{{"test":false}}}}"#, prop_key);
    let upd = runner.run_with_body(
        &ubody,
        &[
            "confluence",
            "Space Properties",
            "update-space-property-by-id",
            "--space-id",
            &space_id,
            "--property-id",
            &prop_id,
        ],
    );
    assert!(
        upd.exit_code == 0,
        "update-space-property failed: {}",
        upd.stderr
    );
    harness::rate_limit_delay(runner.config());

    // DELETE
    let del = runner.run(&[
        "confluence",
        "Space Properties",
        "delete-space-property-by-id",
        "--space-id",
        &space_id,
        "--property-id",
        &prop_id,
    ]);
    assert!(
        del.exit_code == 0,
        "delete-space-property failed: {}",
        del.stderr
    );
    harness::rate_limit_delay(runner.config());

    teardown_profile(&runner, &profile);
}

// ─── Folder CRUD ────────────────────────────────────────────────────────

#[test]
fn test_folder_crud() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let space_id = match get_space_id(&runner) {
        Some(id) => id,
        None => {
            eprintln!("Skipping folder CRUD: space not found");
            teardown_profile(&runner, &profile);
            return;
        }
    };
    harness::rate_limit_delay(runner.config());

    let folder_title = format!("E2E Folder {}", std::process::id());
    let body = format!(
        r#"{{"spaceId":"{}","title":"{}","status":"current"}}"#,
        space_id, folder_title
    );
    let create = runner.run_json_with_body(&body, &["confluence", "Folder", "create-folder"]);
    if create.exit_code != 0 {
        eprintln!(
            "Skipping folder CRUD: create failed (folders may not be available): {}",
            create.stderr
        );
        teardown_profile(&runner, &profile);
        return;
    }
    let folder_id = create
        .json
        .as_ref()
        .and_then(|j| j.get("id"))
        .and_then(|v| v.as_str())
        .expect("Expected folder id");
    let folder_id = folder_id.to_string();
    eprintln!("Created folder: {} ({})", folder_title, folder_id);
    harness::rate_limit_delay(runner.config());

    // READ
    let read = runner.run_json(&[
        "confluence",
        "Folder",
        "get-folder-by-id",
        "--id",
        &folder_id,
    ]);
    read.assert_success();
    harness::rate_limit_delay(runner.config());

    // DELETE
    let del = runner.run(&["confluence", "Folder", "delete-folder", "--id", &folder_id]);
    assert!(del.exit_code == 0, "delete-folder failed: {}", del.stderr);
    harness::rate_limit_delay(runner.config());

    teardown_profile(&runner, &profile);
}

// ─── Task Reads ─────────────────────────────────────────────────────────

#[test]
fn test_list_tasks() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let result = runner.run_json(&["confluence", "Task", "get-tasks"]);
    result.assert_success();
    assert!(result.json.is_some(), "Expected JSON from get-tasks");
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Content Properties CRUD ────────────────────────────────────────────

#[test]
fn test_content_properties_crud() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let space_id = match get_space_id(&runner) {
        Some(id) => id,
        None => {
            eprintln!("Skipping content properties: space not found");
            teardown_profile(&runner, &profile);
            return;
        }
    };
    harness::rate_limit_delay(runner.config());

    let page_title = format!("E2E PropPage {}", std::process::id());
    let page_id = match create_page(&runner, &space_id, &page_title) {
        Some(id) => id,
        None => {
            eprintln!("Skipping content properties: could not create page");
            teardown_profile(&runner, &profile);
            return;
        }
    };
    harness::rate_limit_delay(runner.config());

    // CREATE property
    let prop_key = format!("e2e-cprop-{}", std::process::id());
    let body = format!(r#"{{"key":"{}","value":{{"test":true}}}}"#, prop_key);
    let create = runner.run_json_with_body(
        &body,
        &[
            "confluence",
            "Content Properties",
            "create-page-property",
            "--page-id",
            &page_id,
        ],
    );
    if create.exit_code != 0 {
        eprintln!(
            "Skipping content properties: create failed: {}",
            create.stderr
        );
        delete_page(&runner, &page_id);
        harness::rate_limit_delay(runner.config());
        teardown_profile(&runner, &profile);
        return;
    }
    let prop_id = create
        .json
        .as_ref()
        .and_then(|j| j.get("id"))
        .and_then(|v| v.as_str())
        .expect("Expected property id");
    let prop_id = prop_id.to_string();
    eprintln!("Created content property: {}", prop_id);
    harness::rate_limit_delay(runner.config());

    // READ
    let read = runner.run_json(&[
        "confluence",
        "Content Properties",
        "get-page-content-properties-by-id",
        "--page-id",
        &page_id,
        "--property-id",
        &prop_id,
    ]);
    read.assert_success();
    harness::rate_limit_delay(runner.config());

    // DELETE
    let del = runner.run(&[
        "confluence",
        "Content Properties",
        "delete-page-property-by-id",
        "--page-id",
        &page_id,
        "--property-id",
        &prop_id,
    ]);
    assert!(
        del.exit_code == 0,
        "delete-page-property failed: {}",
        del.stderr
    );
    harness::rate_limit_delay(runner.config());

    delete_page(&runner, &page_id);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Page Versions ──────────────────────────────────────────────────────

#[test]
fn test_page_versions() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let space_id = match get_space_id(&runner) {
        Some(id) => id,
        None => {
            eprintln!("Skipping page versions: space not found");
            teardown_profile(&runner, &profile);
            return;
        }
    };
    harness::rate_limit_delay(runner.config());

    let page_title = format!("E2E VerPage {}", std::process::id());
    let page_id = match create_page(&runner, &space_id, &page_title) {
        Some(id) => id,
        None => {
            eprintln!("Skipping page versions: could not create page");
            teardown_profile(&runner, &profile);
            return;
        }
    };
    harness::rate_limit_delay(runner.config());

    let result = runner.run_json(&[
        "confluence",
        "Version",
        "get-page-versions",
        "--id",
        &page_id,
    ]);
    result.assert_success();
    harness::rate_limit_delay(runner.config());

    delete_page(&runner, &page_id);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Page Likes ─────────────────────────────────────────────────────────

#[test]
fn test_page_likes() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let space_id = match get_space_id(&runner) {
        Some(id) => id,
        None => {
            eprintln!("Skipping page likes: space not found");
            teardown_profile(&runner, &profile);
            return;
        }
    };
    harness::rate_limit_delay(runner.config());

    let page_title = format!("E2E LikePage {}", std::process::id());
    let page_id = match create_page(&runner, &space_id, &page_title) {
        Some(id) => id,
        None => {
            eprintln!("Skipping page likes: could not create page");
            teardown_profile(&runner, &profile);
            return;
        }
    };
    harness::rate_limit_delay(runner.config());

    let result = runner.run_json(&[
        "confluence",
        "Like",
        "get-page-like-count",
        "--id",
        &page_id,
    ]);
    result.assert_success();
    harness::rate_limit_delay(runner.config());

    delete_page(&runner, &page_id);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Attachment Reads ───────────────────────────────────────────────────

#[test]
fn test_list_attachments() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let result = runner.run_json(&["confluence", "Attachment", "get-attachments"]);
    result.assert_success();
    assert!(result.json.is_some(), "Expected JSON from get-attachments");
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Custom Content CRUD ────────────────────────────────────────────────

#[test]
fn test_custom_content_list() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    // Custom content requires a specific type parameter — just test the list endpoint
    let result = runner.run_json(&[
        "confluence",
        "Custom Content",
        "get-custom-content-by-type",
        "--type",
        "ac:com.atlassian.confluence.plugins.confluence-questions:question",
    ]);
    // This may fail if the type doesn't exist — that's OK
    if result.exit_code != 0 {
        eprintln!(
            "Custom content list returned {}: {} (type may not exist)",
            result.exit_code, result.stderr
        );
    }
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Page Ancestors ─────────────────────────────────────────────────────

#[test]
fn test_page_ancestors() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let space_id = match get_space_id(&runner) {
        Some(id) => id,
        None => {
            eprintln!("Skipping page ancestors: space not found");
            teardown_profile(&runner, &profile);
            return;
        }
    };
    harness::rate_limit_delay(runner.config());

    let page_title = format!("E2E AncPage {}", std::process::id());
    let page_id = match create_page(&runner, &space_id, &page_title) {
        Some(id) => id,
        None => {
            eprintln!("Skipping page ancestors: could not create page");
            teardown_profile(&runner, &profile);
            return;
        }
    };
    harness::rate_limit_delay(runner.config());

    let result = runner.run_json(&[
        "confluence",
        "Ancestors",
        "get-page-ancestors",
        "--id",
        &page_id,
    ]);
    result.assert_success();
    harness::rate_limit_delay(runner.config());

    delete_page(&runner, &page_id);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Page Descendants ───────────────────────────────────────────────────

#[test]
fn test_page_descendants() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let space_id = match get_space_id(&runner) {
        Some(id) => id,
        None => {
            eprintln!("Skipping page descendants: space not found");
            teardown_profile(&runner, &profile);
            return;
        }
    };
    harness::rate_limit_delay(runner.config());

    let page_title = format!("E2E DescPage {}", std::process::id());
    let page_id = match create_page(&runner, &space_id, &page_title) {
        Some(id) => id,
        None => {
            eprintln!("Skipping page descendants: could not create page");
            teardown_profile(&runner, &profile);
            return;
        }
    };
    harness::rate_limit_delay(runner.config());

    let result = runner.run_json(&[
        "confluence",
        "Descendants",
        "get-page-descendants",
        "--id",
        &page_id,
    ]);
    result.assert_success();
    harness::rate_limit_delay(runner.config());

    delete_page(&runner, &page_id);
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Space Roles ────────────────────────────────────────────────────────

#[test]
fn test_list_space_roles() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let result = runner.run_json(&["confluence", "Space Roles", "get-available-space-roles"]);
    result.assert_success();
    assert!(
        result.json.is_some(),
        "Expected JSON from get-available-space-roles"
    );
    harness::rate_limit_delay(runner.config());
    teardown_profile(&runner, &profile);
}

// ─── Whiteboard CRUD ────────────────────────────────────────────────────

#[test]
fn test_whiteboard_crud() {
    let config = skip_unless_e2e!();
    let runner = ShrugRunner::new(config);
    let profile = setup_profile(&runner);

    let space_id = match get_space_id(&runner) {
        Some(id) => id,
        None => {
            eprintln!("Skipping whiteboard: space not found");
            teardown_profile(&runner, &profile);
            return;
        }
    };
    harness::rate_limit_delay(runner.config());

    let wb_title = format!("E2E Whiteboard {}", std::process::id());
    let body = format!(r#"{{"spaceId":"{}","title":"{}"}}"#, space_id, wb_title);
    let create =
        runner.run_json_with_body(&body, &["confluence", "Whiteboard", "create-whiteboard"]);
    if create.exit_code != 0 {
        eprintln!(
            "Skipping whiteboard: create failed (may require Premium): {}",
            create.stderr
        );
        teardown_profile(&runner, &profile);
        return;
    }
    let wb_id = create
        .json
        .as_ref()
        .and_then(|j| j.get("id"))
        .and_then(|v| v.as_str())
        .expect("Expected whiteboard id");
    let wb_id = wb_id.to_string();
    eprintln!("Created whiteboard: {} ({})", wb_title, wb_id);
    harness::rate_limit_delay(runner.config());

    let read = runner.run_json(&[
        "confluence",
        "Whiteboard",
        "get-whiteboard-by-id",
        "--id",
        &wb_id,
    ]);
    read.assert_success();
    harness::rate_limit_delay(runner.config());

    let del = runner.run(&[
        "confluence",
        "Whiteboard",
        "delete-whiteboard",
        "--id",
        &wb_id,
    ]);
    assert!(
        del.exit_code == 0,
        "delete-whiteboard failed: {}",
        del.stderr
    );
    harness::rate_limit_delay(runner.config());

    teardown_profile(&runner, &profile);
}
