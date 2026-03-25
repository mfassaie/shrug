//! Mocked E2E tests for Jira Software epic commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_epic_view_sends_get() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/agile/1.0/epic/TEAM-100");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": 100,
                "key": "TEAM-100",
                "name": "Sprint Goals",
                "done": false
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "jira-software", "epic", "view", "TEAM-100"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["key"], "TEAM-100");
}

#[test]
fn test_epic_edit_sends_post() {
    let env = MockEnv::new();

    // Jira Agile uses POST for partial updates (not PUT)
    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/rest/agile/1.0/epic/TEAM-100")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": 100,
                "key": "TEAM-100",
                "name": "Updated Epic",
                "done": false
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "jira-software", "epic", "edit", "TEAM-100",
            "--name", "Updated Epic",
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("Updated") || stdout.contains("TEAM-100"),
        "Expected edit confirmation: {}",
        stdout
    );
}

#[test]
fn test_epic_list_sends_get_issues() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/agile/1.0/epic/TEAM-100/issue");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "issues": [
                    {
                        "key": "TEAM-101",
                        "fields": {
                            "summary": "Task in epic",
                            "status": {"name": "To Do"}
                        }
                    }
                ],
                "total": 1,
                "startAt": 0,
                "maxResults": 50
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "jira-software", "epic", "list", "TEAM-100",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    let arr = json.as_array().expect("Expected JSON array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["key"], "TEAM-101");
}
