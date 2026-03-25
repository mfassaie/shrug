//! Mocked E2E tests for Jira audit log commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_audit_list_sends_get_records() {
    let env = MockEnv::new();

    // The audit API returns "records" as the array field. The paginated getter
    // recognises "issues", "values", "results" but not "records", so the handler
    // produces no output. We verify the correct request path is hit.
    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/auditing/record");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "records": [
                    {
                        "id": 1,
                        "summary": "User logged in",
                        "category": "user management",
                        "created": "2026-03-25T10:00:00.000+0000"
                    }
                ],
                "total": 1,
                "offset": 0,
                "limit": 1000
            }));
    });

    let (_stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["jira", "audit", "list"]),
    );

    mock.assert();
}
