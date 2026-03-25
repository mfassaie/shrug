//! Mocked E2E tests for Confluence page version commands (read-only).

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_page_version_list_sends_get_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/pages/1001/versions");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "results": [
                    {"number": 3, "message": "Updated heading"},
                    {"number": 2, "message": "Added section"},
                    {"number": 1, "message": "Initial version"}
                ],
                "_links": {}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "page", "version", "list", "1001",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    // Non-paginated endpoint returns full response object
    let results = json["results"].as_array().expect("Expected results array");
    assert_eq!(results.len(), 3);
}

#[test]
fn test_page_version_view_sends_get_v2_by_number() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/pages/1001/versions/2");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "number": 2,
                "message": "Added section",
                "createdAt": "2026-03-25T10:00:00.000Z"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "page", "version", "view", "1001", "2",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["number"], 2);
}
