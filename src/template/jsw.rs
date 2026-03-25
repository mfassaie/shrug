//! Jira Software JSON body templates.

use serde_json::{json, Value};

/// Template for `jira-software board create --from-json`.
/// Matches the structure produced by build_create_body in src/jsw/board.rs.
pub fn board_create() -> Value {
    json!({
        "name": "BOARD_NAME",
        "type": "scrum",
        "filterId": 12345
    })
}

/// Template for `jira-software sprint create --from-json`.
/// Matches the structure produced by build_create_body in src/jsw/sprint.rs.
pub fn sprint_create() -> Value {
    json!({
        "name": "SPRINT_NAME",
        "originBoardId": 42,
        "goal": "SPRINT_GOAL_OR_REMOVE",
        "startDate": "2026-01-01T00:00:00.000Z",
        "endDate": "2026-01-14T00:00:00.000Z"
    })
}

/// Template for `jira-software sprint edit --from-json`.
/// Matches the structure produced by build_edit_body in src/jsw/sprint.rs.
pub fn sprint_edit() -> Value {
    json!({
        "name": "SPRINT_NAME",
        "state": "future",
        "goal": "UPDATED_GOAL_OR_REMOVE",
        "startDate": "2026-01-01T00:00:00.000Z",
        "endDate": "2026-01-14T00:00:00.000Z"
    })
}
