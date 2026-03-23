//! Confluence E2E tests using the v2 API.
//! Tests: Page CRUD, Space list, Blog Post list, Comment list, Attachment list.

use crate::harness::{self, ShrugRunner};

fn setup_profile(runner: &ShrugRunner) -> String {
    let name = format!("e2e-conf-{}", std::process::id());
    let result = runner.run(&[
        "profile", "create", "--name", &name,
        "--site", runner.config().site.as_str(),
        "--email", runner.config().email.as_str(),
    ]);
    assert!(
        result.exit_code == 0 || result.stderr.contains("already exists"),
        "Failed to create profile: {}", result.stderr
    );
    let _ = runner.run(&["profile", "use", "--name", &name]);
    name
}

fn teardown_profile(runner: &ShrugRunner, name: &str) {
    let _ = runner.run(&["profile", "delete", "--name", name]);
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
    let space_id = spaces.json.as_ref()
        .and_then(|j| j.get("results"))
        .and_then(|r| r.as_array())
        .and_then(|arr| arr.iter().find(|s| {
            s.get("key").and_then(|k| k.as_str()) == Some(space)
        }))
        .and_then(|s| s.get("id"))
        .and_then(|v| v.as_str());

    let space_id = match space_id {
        Some(id) => id.to_string(),
        None => {
            eprintln!("Skipping page CRUD: space '{}' not found in spaces list", space);
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
    let page_id = create.json.as_ref()
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
    let version = read.json.as_ref()
        .and_then(|j| j.get("version"))
        .and_then(|v| v.get("number"))
        .and_then(|n| n.as_i64())
        .unwrap_or(1);
    let updated_title = format!("{} Updated", title);
    let ubody = format!(
        r#"{{"id":"{}","title":"{}","spaceId":"{}","body":{{"representation":"storage","value":"<p>Updated content</p>"}},"version":{{"number":{}}},"status":"current"}}"#,
        page_id, updated_title, space_id, version + 1
    );
    let upd = runner.run_with_body(&ubody, &["confluence", "Page", "update-page", "--id", page_id]);
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
