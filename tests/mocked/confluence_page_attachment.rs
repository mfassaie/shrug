//! Mocked E2E tests for Confluence page attachment commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_page_attachment_list_sends_get_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/pages/1001/attachments");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "results": [
                    {"id": "att001", "title": "diagram.png", "mediaType": "image/png"}
                ],
                "_links": {}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "page", "attachment", "list", "1001",
        ]),
    );

    mock.assert();
    assert!(stdout.contains("diagram.png") || stdout.contains("att001"), "Expected attachment: {}", stdout);
}

#[test]
fn test_page_attachment_create_sends_multipart_v1() {
    let env = MockEnv::new();

    let tmp = tempfile::NamedTempFile::new().expect("Failed to create temp file");
    std::io::Write::write_all(&mut std::fs::File::create(tmp.path()).unwrap(), b"file data")
        .expect("Failed to write temp file");

    // Attachment create uses v1 multipart with X-Atlassian-Token header
    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/wiki/rest/api/content/1001/child/attachment")
            .header("X-Atlassian-Token", "no-check");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "results": [
                    {"id": "att002", "title": "test.txt", "mediaType": "text/plain"}
                ]
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "page", "attachment", "create", "1001",
            "--file", tmp.path().to_str().unwrap(),
        ]),
    );

    mock.assert();
    assert!(stdout.contains("att002") || stdout.contains("test.txt"), "Expected attachment: {}", stdout);
}

#[test]
fn test_page_attachment_view_sends_get_v2_flat() {
    let env = MockEnv::new();

    // View uses flat v2 path (not parent-nested)
    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/attachments/att001");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "att001",
                "title": "diagram.png",
                "mediaType": "image/png",
                "fileSize": 12345
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "page", "attachment", "view", "att001",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "att001");
}

#[test]
fn test_page_attachment_edit_sends_multipart_post_v1() {
    let env = MockEnv::new();

    let tmp = tempfile::NamedTempFile::new().expect("Failed to create temp file");
    std::io::Write::write_all(&mut std::fs::File::create(tmp.path()).unwrap(), b"new data")
        .expect("Failed to write temp file");

    // Edit (replace file) uses v1 POST (not PUT) with X-Atlassian-Token
    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/wiki/rest/api/content/1001/child/attachment/att001/data")
            .header("X-Atlassian-Token", "no-check");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "att001",
                "title": "diagram.png"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "confluence", "page", "attachment", "edit", "1001", "att001",
            "--file", tmp.path().to_str().unwrap(),
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("Updated") || stdout.contains("att001"),
        "Expected edit confirmation: {}", stdout
    );
}

#[test]
fn test_page_attachment_delete_sends_delete_v2_flat() {
    let env = MockEnv::new();

    // Delete uses flat v2 path
    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/wiki/api/v2/attachments/att001");
        then.status(204);
    });

    helpers::assert_success(
        env.cmd().args([
            "confluence", "page", "attachment", "delete", "att001", "--yes",
        ]),
    );

    mock.assert();
}
