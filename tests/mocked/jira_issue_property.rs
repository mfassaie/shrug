//! Mocked E2E tests for Jira issue property commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_property_list_sends_get() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/issue/TEAM-1/properties");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "keys": [
                    {"key": "my.custom.prop", "self": "https://example.atlassian.net/rest/api/3/issue/TEAM-1/properties/my.custom.prop"}
                ]
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "jira", "issue", "property", "list", "TEAM-1"]),
    );

    mock.assert();
    assert!(
        stdout.contains("my.custom.prop"),
        "Expected property key in output: {}",
        stdout
    );
}

#[test]
fn test_property_view_sends_get_by_key() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/issue/TEAM-1/properties/my.custom.prop");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "key": "my.custom.prop",
                "value": {"count": 42, "enabled": true}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "jira", "issue", "property", "view", "TEAM-1", "my.custom.prop",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["key"], "my.custom.prop");
}

#[test]
fn test_property_edit_sends_put() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(PUT)
            .path("/rest/api/3/issue/TEAM-1/properties/my.custom.prop")
            .header("content-type", "application/json");
        then.status(200);
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "jira", "issue", "property", "edit", "TEAM-1", "my.custom.prop",
            "--value", r#"{"count":99}"#,
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("Updated") || stdout.contains("Set") || stdout.contains("my.custom.prop"),
        "Expected edit confirmation: {}",
        stdout
    );
}

#[test]
fn test_property_delete_sends_delete() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/rest/api/3/issue/TEAM-1/properties/my.custom.prop");
        then.status(204);
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "jira", "issue", "property", "delete", "TEAM-1", "my.custom.prop", "--yes",
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("Deleted") || stdout.contains("my.custom.prop"),
        "Expected delete confirmation: {}",
        stdout
    );
}
