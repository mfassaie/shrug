//! Mocked E2E tests for Confluence page comment commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_page_comment_list_sends_get_footer_comments() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/pages/1001/footer-comments");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "results": [
                    {"id": "80001", "body": {"storage": {"value": "<p>Nice work</p>"}}}
                ],
                "_links": {}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "page", "comment", "list", "1001",
        ]),
    );

    mock.assert();
    assert!(stdout.contains("80001"), "Expected comment ID: {}", stdout);
}

#[test]
fn test_page_comment_create_sends_post_footer() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/wiki/api/v2/footer-comments")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "80002",
                "body": {"storage": {"value": "<p>Comment text</p>"}}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "page", "comment", "create", "1001",
            "--body", "Comment text",
        ]),
    );

    mock.assert();
    assert!(stdout.contains("80002"), "Expected comment ID: {}", stdout);
}

#[test]
fn test_page_comment_view_sends_get_by_id() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/footer-comments/80001");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "80001",
                "body": {"storage": {"value": "<p>Nice work</p>"}},
                "version": {"number": 1}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "page", "comment", "view", "80001",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "80001");
}

#[test]
fn test_page_comment_edit_fetches_version_then_puts() {
    let env = MockEnv::new();

    let get_mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/footer-comments/80001");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "80001",
                "body": {"storage": {"value": "<p>Old</p>"}},
                "version": {"number": 1}
            }));
    });

    let put_mock = env.server.mock(|when, then| {
        when.method(PUT)
            .path("/wiki/api/v2/footer-comments/80001")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "80001",
                "version": {"number": 2}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "confluence", "page", "comment", "edit", "80001",
            "--body", "Updated comment",
        ]),
    );

    get_mock.assert();
    put_mock.assert();
    assert!(
        stdout.contains("Updated") || stdout.contains("80001"),
        "Expected edit confirmation: {}", stdout
    );
}

#[test]
fn test_page_comment_delete_sends_delete() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/wiki/api/v2/footer-comments/80001");
        then.status(204);
    });

    helpers::assert_success(
        env.cmd().args([
            "confluence", "page", "comment", "delete", "80001", "--yes",
        ]),
    );

    mock.assert();
}
