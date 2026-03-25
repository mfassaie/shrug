//! Mocked E2E tests for Confluence search commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_search_list_sends_get_v1_with_cql() {
    let env = MockEnv::new();

    // Confluence search uses v1 API: /wiki/rest/api/search with cql query param
    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/rest/api/search");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "results": [
                    {
                        "content": {
                            "id": "1001",
                            "type": "page",
                            "title": "Getting Started"
                        }
                    }
                ],
                "totalSize": 1,
                "start": 0,
                "limit": 25,
                "size": 1
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "search", "list",
            "--cql", "type = page",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    let arr = json.as_array().expect("Expected JSON array");
    assert_eq!(arr.len(), 1);
}
