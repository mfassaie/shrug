//! Mocked E2E tests for Jira project version commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_version_list_sends_get_by_project() {
    let env = MockEnv::new();

    // List uses project-nested path: /rest/api/3/project/{key}/version
    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/project/TEAM/version");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "values": [
                    {
                        "id": "70000",
                        "name": "v1.0",
                        "released": true,
                        "archived": false
                    },
                    {
                        "id": "70001",
                        "name": "v2.0",
                        "released": false,
                        "archived": false
                    }
                ],
                "total": 2,
                "isLast": true
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "jira", "project", "version", "list", "TEAM",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    let arr = json.as_array().expect("Expected JSON array");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["name"], "v1.0");
}

#[test]
fn test_version_create_sends_post() {
    let env = MockEnv::new();

    // Create uses flat path: /rest/api/3/version
    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/rest/api/3/version")
            .header("content-type", "application/json");
        then.status(201)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "70002",
                "name": "v3.0",
                "self": "https://example.atlassian.net/rest/api/3/version/70002"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "jira", "project", "version", "create",
            "--name", "v3.0",
            "--project", "TEAM",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "70002");
}

#[test]
fn test_version_view_sends_get_by_id() {
    let env = MockEnv::new();

    // View uses flat path: /rest/api/3/version/{id}
    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/version/70000");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "70000",
                "name": "v1.0",
                "released": true,
                "archived": false,
                "projectId": 10000
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "jira", "project", "version", "view", "70000",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "70000");
    assert_eq!(json["name"], "v1.0");
}

#[test]
fn test_version_edit_sends_put() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(PUT)
            .path("/rest/api/3/version/70000")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "70000",
                "name": "v1.0.1"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "jira", "project", "version", "edit", "70000",
            "--name", "v1.0.1",
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("Updated") || stdout.contains("70000"),
        "Expected edit confirmation: {}",
        stdout
    );
}

#[test]
fn test_version_delete_sends_delete() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/rest/api/3/version/70000");
        then.status(204);
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "jira", "project", "version", "delete", "70000", "--yes",
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("Deleted") || stdout.contains("70000"),
        "Expected delete confirmation: {}",
        stdout
    );
}
