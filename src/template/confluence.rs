//! Confluence JSON body templates.

use serde_json::{json, Value};

/// Template for `confluence space create --from-json`.
/// Matches the structure produced by build_create_body in src/confluence/space/mod.rs.
pub fn space_create() -> Value {
    json!({
        "key": "SPACEKEY",
        "name": "SPACE_NAME",
        "description": {
            "plain": {
                "value": "SPACE_DESCRIPTION_OR_REMOVE",
                "representation": "plain"
            }
        },
        "type": "global"
    })
}

/// Template for `confluence space edit --from-json`.
/// Matches the structure produced by build_edit_body in src/confluence/space/mod.rs (v1 API).
pub fn space_edit() -> Value {
    json!({
        "name": "UPDATED_NAME",
        "description": {
            "plain": {
                "value": "UPDATED_DESCRIPTION_OR_REMOVE",
                "representation": "plain"
            }
        }
    })
}

/// Template for `confluence page create --from-json`.
/// Matches the structure produced by build_create_body in src/confluence/page/mod.rs.
pub fn page_create() -> Value {
    json!({
        "spaceId": "SPACE_ID",
        "title": "PAGE_TITLE",
        "status": "current",
        "body": {
            "representation": "storage",
            "value": "<p>PAGE_CONTENT</p>"
        },
        "parentId": "PARENT_ID_OR_REMOVE"
    })
}

/// Template for `confluence page edit --from-json`.
/// Matches the structure produced by build_edit_body in src/confluence/page/mod.rs.
/// The version number is auto-incremented by the handler if not present.
pub fn page_edit() -> Value {
    json!({
        "id": "PAGE_ID",
        "title": "UPDATED_TITLE",
        "status": "current",
        "body": {
            "representation": "storage",
            "value": "<p>UPDATED_CONTENT</p>"
        },
        "version": {
            "number": 2,
            "message": "VERSION_MESSAGE_OR_REMOVE"
        }
    })
}

/// Template for `confluence blogpost create --from-json`.
/// Matches the structure produced by build_create_body in src/confluence/blogpost/mod.rs.
pub fn blogpost_create() -> Value {
    json!({
        "spaceId": "SPACE_ID",
        "title": "BLOGPOST_TITLE",
        "status": "current",
        "body": {
            "representation": "storage",
            "value": "<p>BLOGPOST_CONTENT</p>"
        }
    })
}

/// Template for `confluence blogpost edit --from-json`.
/// Matches the structure produced by build_edit_body in src/confluence/blogpost/mod.rs.
/// The version number is auto-incremented by the handler if not present.
pub fn blogpost_edit() -> Value {
    json!({
        "id": "BLOGPOST_ID",
        "title": "UPDATED_TITLE",
        "status": "current",
        "body": {
            "representation": "storage",
            "value": "<p>UPDATED_CONTENT</p>"
        },
        "version": {
            "number": 2,
            "message": "VERSION_MESSAGE_OR_REMOVE"
        }
    })
}

/// Template for `confluence custom-content create --from-json`.
/// Matches the structure produced by build_create_body in src/confluence/custom_content.rs.
pub fn custom_content_create() -> Value {
    json!({
        "type": "ac:APP_KEY:CONTENT_TYPE",
        "title": "TITLE",
        "spaceId": "SPACE_ID",
        "body": {
            "representation": "storage",
            "value": "<p>CUSTOM_CONTENT</p>"
        },
        "pageId": "PARENT_PAGE_ID_OR_REMOVE"
    })
}

/// Template for `confluence custom-content edit --from-json`.
/// Matches the structure produced by build_edit_body in src/confluence/custom_content.rs.
/// The version number is auto-incremented by the handler if not present.
pub fn custom_content_edit() -> Value {
    json!({
        "id": "CONTENT_ID",
        "title": "UPDATED_TITLE",
        "body": {
            "representation": "storage",
            "value": "<p>UPDATED_CONTENT</p>"
        },
        "version": {
            "number": 2,
            "message": "VERSION_MESSAGE_OR_REMOVE"
        }
    })
}
