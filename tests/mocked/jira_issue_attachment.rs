//! Mocked E2E tests for Jira issue attachment commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_attachment_list_sends_get_issue_with_fields() {
    let env = MockEnv::new();

    // Attachment list fetches the issue with ?fields=attachment
    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/issue/TEAM-1")
            .query_param("fields", "attachment");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "key": "TEAM-1",
                "fields": {
                    "attachment": [
                        {
                            "id": "30000",
                            "filename": "screenshot.png",
                            "size": 12345,
                            "mimeType": "image/png"
                        }
                    ]
                }
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "jira", "issue", "attachment", "list", "TEAM-1"]),
    );

    mock.assert();
    assert!(
        stdout.contains("screenshot.png") || stdout.contains("30000"),
        "Expected attachment info in output: {}",
        stdout
    );
}

#[test]
fn test_attachment_create_sends_multipart_post() {
    let env = MockEnv::new();

    // Create a temp file to upload
    let tmp = tempfile::NamedTempFile::new().expect("Failed to create temp file");
    std::io::Write::write_all(&mut std::fs::File::create(tmp.path()).unwrap(), b"test content")
        .expect("Failed to write temp file");

    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/rest/api/3/issue/TEAM-1/attachments")
            .header("X-Atlassian-Token", "no-check");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!([
                {
                    "id": "30001",
                    "filename": "test.txt",
                    "size": 12,
                    "self": "https://example.atlassian.net/rest/api/3/attachment/30001"
                }
            ]));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "jira", "issue", "attachment", "create", "TEAM-1",
            "--file", tmp.path().to_str().unwrap(),
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("30001") || stdout.contains("test.txt"),
        "Expected attachment info in output: {}",
        stdout
    );
}

#[test]
fn test_attachment_view_sends_get_by_id() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/attachment/30000");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "30000",
                "filename": "screenshot.png",
                "size": 12345,
                "mimeType": "image/png"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "jira", "issue", "attachment", "view", "30000"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "30000");
}

#[test]
fn test_attachment_delete_sends_delete() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/rest/api/3/attachment/30000");
        then.status(204);
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "jira", "issue", "attachment", "delete", "30000", "--yes",
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("Deleted") || stdout.contains("30000"),
        "Expected delete confirmation: {}",
        stdout
    );
}
