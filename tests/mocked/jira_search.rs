//! Mocked E2E tests for Jira search commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_search_list_sends_get_jql() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/search/jql");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "issues": [
                    {
                        "key": "TEAM-1",
                        "fields": {
                            "summary": "Found issue",
                            "status": {"name": "Open"}
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
            "jira", "search", "list",
            "--jql", "project = TEAM",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    let arr = json.as_array().expect("Expected JSON array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["key"], "TEAM-1");
}
