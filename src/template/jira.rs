//! Jira JSON body templates.

use serde_json::{json, Value};

/// Template for `jira issue create --from-json`.
/// Matches the structure produced by build_create_body in src/jira/issue/mod.rs.
pub fn issue_create() -> Value {
    json!({
        "fields": {
            "summary": "YOUR_SUMMARY",
            "project": {"key": "PROJECT_KEY"},
            "issuetype": {"name": "Task"},
            "description": {
                "type": "doc",
                "version": 1,
                "content": [
                    {
                        "type": "paragraph",
                        "content": [
                            {"type": "text", "text": "YOUR_DESCRIPTION"}
                        ]
                    }
                ]
            },
            "assignee": {"accountId": "ACCOUNT_ID_OR_REMOVE"},
            "reporter": {"accountId": "ACCOUNT_ID_OR_REMOVE"},
            "priority": {"name": "Medium"},
            "labels": ["label1", "label2"],
            "components": [{"name": "component1"}],
            "fixVersions": [{"name": "v1.0"}],
            "parent": {"key": "PARENT_KEY_OR_REMOVE"},
            "duedate": "2026-12-31"
        }
    })
}

/// Template for `jira issue edit --from-json`.
/// Matches the structure produced by build_edit_body in src/jira/issue/mod.rs.
pub fn issue_edit() -> Value {
    json!({
        "fields": {
            "summary": "UPDATED_SUMMARY",
            "description": {
                "type": "doc",
                "version": 1,
                "content": [
                    {
                        "type": "paragraph",
                        "content": [
                            {"type": "text", "text": "UPDATED_DESCRIPTION"}
                        ]
                    }
                ]
            },
            "assignee": {"accountId": "ACCOUNT_ID_OR_REMOVE"},
            "priority": {"name": "High"},
            "parent": {"key": "PARENT_KEY_OR_REMOVE"},
            "duedate": "2026-12-31"
        },
        "update": {
            "labels": [
                {"add": "new-label"},
                {"remove": "old-label"}
            ],
            "components": [
                {"add": {"name": "new-component"}},
                {"remove": {"name": "old-component"}}
            ],
            "fixVersions": [
                {"add": {"name": "v2.0"}},
                {"remove": {"name": "v1.0"}}
            ]
        }
    })
}
