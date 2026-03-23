//! Live API CRUD smoke tests — Jira issue and Confluence page lifecycle.
//!
//! These tests require E2E credentials AND shrug on PATH.
//! They are skipped if either is missing.
//!
//! Run with: cargo test --test smoke live_api -- --test-threads=1

use crate::harness::{self, SmokeRunner};

/// Generate a unique profile name.
fn unique_name(prefix: &str) -> String {
    format!("la-{}-{}", prefix, std::process::id())
}

/// Create a temp profile with real credentials and set as default.
fn setup_profile(runner: &SmokeRunner, e2e: &crate::harness::E2eConfig) -> String {
    let name = unique_name("live");
    let result = runner.run(&[
        "profile", "create", "--name", &name, "--site", &e2e.site, "--email", &e2e.email,
    ]);
    assert!(
        result.exit_code == 0 || result.stderr.contains("already exists"),
        "Failed to create profile '{}': {}",
        name,
        result.stderr
    );
    let _ = runner.run(&["profile", "use", "--name", &name]);
    name
}

/// Delete a temp profile (best-effort).
fn teardown_profile(runner: &SmokeRunner, name: &str) {
    let _ = runner.run(&["profile", "delete", "--name", name]);
}

// ─── Jira Issue CRUD ─────────────────────────────────────────────────────

#[test]
fn test_jira_issue_create_and_delete() {
    let (smoke_config, e2e_config) = skip_unless_e2e!();
    let runner = SmokeRunner::with_e2e(smoke_config, e2e_config.clone());
    let profile = setup_profile(&runner, &e2e_config);

    // Create issue
    let result = runner.run_json(&[
        "jira",
        "+create",
        "--project",
        &e2e_config.jira_project,
        "--summary",
        "Smoke test issue (auto-delete)",
    ]);
    if result.exit_code != 0 {
        eprintln!("Skipping Jira CRUD: +create failed: {}", result.stderr);
        teardown_profile(&runner, &profile);
        return;
    }

    let key = result
        .json
        .as_ref()
        .and_then(|j| j.get("key"))
        .and_then(|v| v.as_str())
        .expect("Expected issue key from +create");
    eprintln!("Created issue: {}", key);
    harness::rate_limit_delay(&e2e_config);

    // Verify issue exists
    let get = runner.run_json(&[
        "jira",
        "Issues",
        "get-issue",
        &format!("--issueIdOrKey={}", key),
    ]);
    get.assert_success();
    assert!(
        get.json
            .as_ref()
            .and_then(|j| j.get("key"))
            .and_then(|v| v.as_str())
            == Some(key),
        "GET should return the same issue key"
    );
    harness::rate_limit_delay(&e2e_config);

    // Delete issue
    let del = runner.run(&[
        "jira",
        "Issues",
        "delete-issue",
        &format!("--issueIdOrKey={}", key),
    ]);
    if del.exit_code == 0 {
        eprintln!("Deleted issue: {}", key);
    } else {
        eprintln!("Warning: failed to delete issue '{}': {}", key, del.stderr);
    }
    harness::rate_limit_delay(&e2e_config);

    teardown_profile(&runner, &profile);
}

// ─── Confluence Page CRUD ────────────────────────────────────────────────

#[test]
fn test_confluence_page_create_and_delete() {
    let (smoke_config, e2e_config) = skip_unless_e2e!();
    let runner = SmokeRunner::with_e2e(smoke_config, e2e_config.clone());
    let profile = setup_profile(&runner, &e2e_config);

    // Look up space ID
    let spaces = runner.run_json(&["confluence", "Space", "get-spaces"]);
    if spaces.exit_code != 0 {
        eprintln!(
            "Skipping Confluence CRUD: get-spaces failed: {}",
            spaces.stderr
        );
        teardown_profile(&runner, &profile);
        return;
    }
    harness::rate_limit_delay(&e2e_config);

    let space_key = &e2e_config.confluence_space;
    let space_id = spaces
        .json
        .as_ref()
        .and_then(|j| j.get("results"))
        .and_then(|r| r.as_array())
        .and_then(|arr| {
            arr.iter()
                .find(|s| s.get("key").and_then(|k| k.as_str()) == Some(space_key.as_str()))
        })
        .and_then(|s| s.get("id"))
        .and_then(|v| v.as_str());

    let space_id = match space_id {
        Some(id) => id.to_string(),
        None => {
            eprintln!(
                "Skipping Confluence CRUD: space '{}' not found in spaces list",
                space_key
            );
            teardown_profile(&runner, &profile);
            return;
        }
    };

    // Create page
    let title = format!("Smoke test page {}", std::process::id());
    let body = format!(
        r#"{{"spaceId":"{}","title":"{}","body":{{"representation":"storage","value":"<p>Smoke test content</p>"}},"status":"current"}}"#,
        space_id, title
    );
    let create = runner.run_json_with_body(&body, &["confluence", "Page", "create-page"]);
    if create.exit_code != 0 {
        eprintln!(
            "Skipping Confluence CRUD: create-page failed: {}",
            create.stderr
        );
        teardown_profile(&runner, &profile);
        return;
    }

    let page_id = create
        .json
        .as_ref()
        .and_then(|j| j.get("id"))
        .and_then(|v| v.as_str())
        .expect("Expected page id from create-page")
        .to_string();
    eprintln!("Created page: {} ({})", title, page_id);
    harness::rate_limit_delay(&e2e_config);

    // Verify page exists
    let get = runner.run_json(&["confluence", "Page", "get-page-by-id", "--id", &page_id]);
    get.assert_success();
    harness::rate_limit_delay(&e2e_config);

    // Delete page
    let del = runner.run(&["confluence", "Page", "delete-page", "--id", &page_id]);
    if del.exit_code == 0 {
        eprintln!("Deleted page: {}", page_id);
    } else {
        eprintln!(
            "Warning: failed to delete page '{}': {}",
            page_id, del.stderr
        );
    }
    harness::rate_limit_delay(&e2e_config);

    teardown_profile(&runner, &profile);
}
