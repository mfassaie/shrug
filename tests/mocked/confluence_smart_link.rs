//! Mocked E2E tests for Confluence smart link (embed) commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_smart_link_create_sends_post_v2_embeds() {
    let env = MockEnv::new();

    // CLI: confluence smart-link create <url>
    // API: POST /wiki/api/v2/embeds
    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/wiki/api/v2/embeds")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "7001",
                "title": "Linked Page",
                "spaceId": "12345"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "smart-link", "create",
            "https://example.com/doc",
            "--space-id", "12345",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "7001");
}

#[test]
fn test_smart_link_view_sends_get_v2_embeds() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/embeds/7001");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "7001",
                "title": "Linked Page",
                "spaceId": "12345"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "confluence", "smart-link", "view", "7001"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "7001");
}

#[test]
fn test_smart_link_delete_sends_delete_v2_embeds() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/wiki/api/v2/embeds/7001");
        then.status(204);
    });

    helpers::assert_success(
        env.cmd()
            .args(["confluence", "smart-link", "delete", "7001", "--yes"]),
    );

    mock.assert();
}
