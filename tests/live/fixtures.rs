//! Test fixture lifecycle: resource tracking and cleanup.
//!
//! ResourceTracker implements Drop for panic-safe cleanup — if a test panics
//! after creating resources, tracked resources are still cleaned up.

use crate::harness::{E2eConfig, ShrugRunner};

/// A resource created during a test that needs cleanup.
struct TrackedResource {
    resource_type: String,
    id: String,
    cleanup_args: Vec<String>,
}

/// Tracks resources created during E2E tests and cleans them up on drop.
///
/// Resources are cleaned in reverse order (LIFO) so dependent resources
/// are removed before their parents.
#[allow(dead_code)]
pub struct ResourceTracker {
    resources: Vec<TrackedResource>,
    config: E2eConfig,
}

#[allow(dead_code)]
impl ResourceTracker {
    /// Create a new tracker that will use the given config for cleanup.
    pub fn new(config: E2eConfig) -> Self {
        Self {
            resources: Vec::new(),
            config,
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
            "--- E2E cleanup: {} resources to remove ---",
            resources.len()
        );
        let runner = ShrugRunner::new(self.config.clone());

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
        eprintln!("--- E2E cleanup complete ---");
    }
}

impl Drop for ResourceTracker {
    fn drop(&mut self) {
        self.cleanup();
    }
}

/// Create a Jira test issue using static CLI and track it for cleanup.
///
/// Returns the parsed JSON response (contains "key", "id", "self" fields).
#[allow(dead_code)]
pub fn create_test_issue(
    runner: &ShrugRunner,
    tracker: &mut ResourceTracker,
    project_key: &str,
    summary: &str,
) -> serde_json::Value {
    let result = runner.run_json(&[
        "jira", "issue", "create",
        "-s", summary,
        "--project", project_key,
        "--type", "Task",
    ]);
    result.assert_success();

    let json = result
        .json
        .expect("Expected JSON response from jira issue create");

    if let Some(key) = json.get("key").and_then(|k| k.as_str()) {
        tracker.track(
            "jira-issue",
            key,
            vec![
                "jira".into(),
                "issue".into(),
                "delete".into(),
                key.into(),
                "--yes".into(),
            ],
        );
    }

    json
}

/// Delete a Jira issue by key using static CLI.
#[allow(dead_code)]
pub fn delete_test_issue(runner: &ShrugRunner, issue_key: &str) {
    let result = runner.run(&["jira", "issue", "delete", issue_key, "--yes"]);
    if result.exit_code != 0 {
        eprintln!(
            "Warning: failed to delete issue '{}': {}",
            issue_key, result.stderr
        );
    }
}
