//! Mocked E2E tests for Confluence blogpost commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_blogpost_list_sends_get_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/blogposts");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "results": [
                    {
                        "id": "2001",
                        "title": "Release Notes",
                        "status": "current",
                        "spaceId": "12345"
                    }
                ],
                "_links": {}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "confluence", "blogpost", "list"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    let arr = json.as_array().expect("Expected JSON array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["title"], "Release Notes");
}

#[test]
fn test_blogpost_create_sends_post_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/wiki/api/v2/blogposts")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "2002",
                "title": "New Post",
                "status": "current",
                "spaceId": "12345"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "blogpost", "create",
            "--title", "New Post",
            "--space-id", "12345",
            "--body", "Post content here",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "2002");
}

#[test]
fn test_blogpost_view_sends_get_v2_by_id() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/blogposts/2001");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "2001",
                "title": "Release Notes",
                "status": "current",
                "spaceId": "12345",
                "version": {"number": 1}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "confluence", "blogpost", "view", "2001"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "2001");
    assert_eq!(json["title"], "Release Notes");
}

#[test]
fn test_blogpost_edit_fetches_version_then_puts() {
    let env = MockEnv::new();

    // GET to fetch current version
    let get_mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/blogposts/2001");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "2001",
                "title": "Release Notes",
                "status": "current",
                "spaceId": "12345",
                "version": {"number": 2}
            }));
    });

    // PUT with incremented version
    let put_mock = env.server.mock(|when, then| {
        when.method(PUT)
            .path("/wiki/api/v2/blogposts/2001")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "2001",
                "title": "Updated Release Notes",
                "status": "current",
                "version": {"number": 3}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "blogpost", "edit", "2001",
            "--title", "Updated Release Notes",
        ]),
    );

    get_mock.assert();
    put_mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["status"], "updated");
}

#[test]
fn test_blogpost_delete_sends_delete_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/wiki/api/v2/blogposts/2001");
        then.status(204);
    });

    helpers::assert_success(
        env.cmd()
            .args(["confluence", "blogpost", "delete", "2001", "--yes"]),
    );

    mock.assert();
}
