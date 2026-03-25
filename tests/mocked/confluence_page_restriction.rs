//! Mocked E2E tests for Confluence page restriction commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_page_restriction_view_sends_get_v1() {
    let env = MockEnv::new();

    // Restriction uses generic v1 content path
    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/rest/api/content/1001/restriction");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "results": [
                    {
                        "operation": "read",
                        "restrictions": {
                            "user": {"results": [{"accountId": "u1", "displayName": "Alice"}]},
                            "group": {"results": []}
                        }
                    }
                ]
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "page", "restriction", "view", "1001",
        ]),
    );

    mock.assert();
    assert!(stdout.contains("read") || stdout.contains("Alice"), "Expected restriction info: {}", stdout);
}

#[test]
fn test_page_restriction_edit_sends_put_v1() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(PUT)
            .path("/wiki/rest/api/content/1001/restriction")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "results": [
                    {
                        "operation": "read",
                        "restrictions": {
                            "user": {"results": [{"accountId": "u1"}]},
                            "group": {"results": []}
                        }
                    }
                ]
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "confluence", "page", "restriction", "edit", "1001", "read",
            "--user", "u1",
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("Updated") || stdout.contains("restriction") || stdout.contains("read"),
        "Expected edit confirmation: {}", stdout
    );
}

#[test]
fn test_page_restriction_delete_sends_delete_v1() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/wiki/rest/api/content/1001/restriction");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({}));
    });

    helpers::assert_success(
        env.cmd().args([
            "confluence", "page", "restriction", "delete", "1001", "--yes",
        ]),
    );

    mock.assert();
}
