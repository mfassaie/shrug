//! Mocked E2E tests for Jira project component commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_component_list_sends_get_by_project() {
    let env = MockEnv::new();

    // List uses project-nested path: /rest/api/3/project/{key}/component
    // Component list returns a bare JSON array (not paginated)
    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/project/TEAM/component");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!([
                {
                    "id": "60000",
                    "name": "Backend",
                    "project": "TEAM"
                },
                {
                    "id": "60001",
                    "name": "Frontend",
                    "project": "TEAM"
                }
            ]));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "jira", "project", "component", "list", "TEAM",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    let arr = json.as_array().expect("Expected JSON array");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["name"], "Backend");
}

#[test]
fn test_component_create_sends_post() {
    let env = MockEnv::new();

    // Create uses flat path: /rest/api/3/component
    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/rest/api/3/component")
            .header("content-type", "application/json");
        then.status(201)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "60002",
                "name": "API",
                "project": "TEAM",
                "self": "https://example.atlassian.net/rest/api/3/component/60002"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "jira", "project", "component", "create",
            "--name", "API",
            "--project", "TEAM",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "60002");
}

#[test]
fn test_component_view_sends_get_by_id() {
    let env = MockEnv::new();

    // View uses flat path: /rest/api/3/component/{id}
    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/component/60000");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "60000",
                "name": "Backend",
                "project": "TEAM"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "jira", "project", "component", "view", "60000",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "60000");
    assert_eq!(json["name"], "Backend");
}

#[test]
fn test_component_edit_sends_put() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(PUT)
            .path("/rest/api/3/component/60000")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "60000",
                "name": "Backend Services"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "jira", "project", "component", "edit", "60000",
            "--name", "Backend Services",
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("Updated") || stdout.contains("60000"),
        "Expected edit confirmation: {}",
        stdout
    );
}

#[test]
fn test_component_delete_sends_delete() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/rest/api/3/component/60000");
        then.status(204);
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "jira", "project", "component", "delete", "60000", "--yes",
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("Deleted") || stdout.contains("60000"),
        "Expected delete confirmation: {}",
        stdout
    );
}
