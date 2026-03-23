//! Test fixture lifecycle: resource tracking and cleanup for smoke tests.
//!
//! ResourceTracker implements Drop for panic-safe cleanup — if a test panics
//! after creating resources, tracked resources are still cleaned up.

use crate::harness::{E2eConfig, SmokeConfig, SmokeRunner};

/// A resource created during a test that needs cleanup.
struct TrackedResource {
    resource_type: String,
    id: String,
    cleanup_args: Vec<String>,
}

/// Tracks resources created during smoke tests and cleans them up on drop.
///
/// Resources are cleaned in reverse order (LIFO) so dependent resources
/// are removed before their parents.
#[allow(dead_code)]
pub struct ResourceTracker {
    resources: Vec<TrackedResource>,
    config: SmokeConfig,
    e2e: E2eConfig,
}

#[allow(dead_code)]
impl ResourceTracker {
    /// Create a new tracker that will use the given config for cleanup.
    pub fn new(config: SmokeConfig, e2e: E2eConfig) -> Self {
        Self {
            resources: Vec::new(),
            config,
            e2e,
        }
    }

    /// Record a resource for later cleanup.
    ///
    /// - `resource_type`: Human-readable type (e.g., "jira-issue", "confluence-page")
    /// - `id`: Resource identifier (e.g., "TEST-123")
    /// - `cleanup_args`: Full CLI args to delete/remove this resource
    pub fn track(&mut self, resource_type: &str, id: &str, cleanup_args: Vec<String>) {
        self.resources.push(TrackedResource {
            resource_type: resource_type.to_string(),
            id: id.to_string(),
            cleanup_args,
        });
    }

    /// Clean up all tracked resources in reverse order.
    /// Drains the resource list so subsequent calls are no-ops.
    /// Logs all actions and continues on failure (best-effort).
    pub fn cleanup(&mut self) {
        let resources: Vec<TrackedResource> = self.resources.drain(..).collect();
        if resources.is_empty() {
            return;
        }

        eprintln!(
            "--- Smoke cleanup: {} resources to remove ---",
            resources.len()
        );
        let runner = SmokeRunner::with_e2e(self.config.clone(), self.e2e.clone());

        for resource in resources.iter().rev() {
            eprintln!(
                "Cleaning up {} '{}'...",
                resource.resource_type, resource.id
            );
            let args: Vec<&str> = resource.cleanup_args.iter().map(|s| s.as_str()).collect();
            let result = runner.run(&args);
            if result.exit_code != 0 {
                eprintln!(
                    "Warning: cleanup failed for {} '{}' (exit {}): {}",
                    resource.resource_type, resource.id, result.exit_code, result.stderr
                );
            } else {
                eprintln!("Cleaned up {} '{}'", resource.resource_type, resource.id);
            }
        }
        eprintln!("--- Smoke cleanup complete ---");
    }
}

impl Drop for ResourceTracker {
    fn drop(&mut self) {
        self.cleanup();
    }
}

/// Create a Jira test issue and track it for cleanup.
///
/// Returns the parsed JSON response (contains "key", "id", "self" fields).
#[allow(dead_code)]
pub fn create_test_issue(
    runner: &SmokeRunner,
    tracker: &mut ResourceTracker,
    project_key: &str,
    summary: &str,
) -> serde_json::Value {
    let result = runner.run_json(&[
        "jira",
        "+create",
        "--project",
        project_key,
        "--summary",
        summary,
    ]);
    result.assert_success();

    let json = result
        .json
        .expect("Expected JSON response from jira +create");

    if let Some(key) = json.get("key").and_then(|k| k.as_str()) {
        tracker.track(
            "jira-issue",
            key,
            vec![
                "jira".into(),
                "issues".into(),
                "deleteIssue".into(),
                format!("--issueIdOrKey={}", key),
            ],
        );
    }

    json
}

/// Delete a Jira issue by key.
#[allow(dead_code)]
pub fn delete_test_issue(runner: &SmokeRunner, issue_key: &str) {
    let result = runner.run(&[
        "jira",
        "issues",
        "deleteIssue",
        &format!("--issueIdOrKey={}", issue_key),
    ]);
    if result.exit_code != 0 {
        eprintln!(
            "Warning: failed to delete issue '{}': {}",
            issue_key, result.stderr
        );
    }
}

/// Create a Confluence test page and track it for cleanup.
///
/// Returns the parsed JSON response (contains "id", "title" fields).
#[allow(dead_code)]
pub fn create_test_page(
    runner: &SmokeRunner,
    tracker: &mut ResourceTracker,
    space_key: &str,
    title: &str,
) -> serde_json::Value {
    let body = serde_json::json!({
        "spaceId": space_key,
        "title": title,
        "status": "current",
        "body": {
            "representation": "storage",
            "value": "<p>Smoke test page</p>"
        }
    });

    let result =
        runner.run_json_with_body(&body.to_string(), &["confluence", "pages", "createPage"]);
    result.assert_success();

    let json = result
        .json
        .expect("Expected JSON response from confluence pages createPage");

    if let Some(id) = json
        .get("id")
        .and_then(|v| v.as_str().or_else(|| v.as_i64().map(|_| "")))
    {
        let id_str = json
            .get("id")
            .map(|v| v.to_string().trim_matches('"').to_string())
            .unwrap_or_default();
        tracker.track(
            "confluence-page",
            &id_str,
            vec![
                "confluence".into(),
                "pages".into(),
                "deletePage".into(),
                format!("--id={}", id_str),
            ],
        );
        let _ = id; // suppress unused
    }

    json
}

/// Delete a Confluence page by ID.
#[allow(dead_code)]
pub fn delete_test_page(runner: &SmokeRunner, page_id: &str) {
    let result = runner.run(&[
        "confluence",
        "pages",
        "deletePage",
        &format!("--id={}", page_id),
    ]);
    if result.exit_code != 0 {
        eprintln!(
            "Warning: failed to delete page '{}': {}",
            page_id, result.stderr
        );
    }
}
