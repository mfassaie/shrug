//! JQL shorthand builder for common Jira search patterns.
//!
//! Converts shorthand flags (--project, --assignee, --status, etc.) into
//! JQL query strings. Supports combining shorthand with raw --jql queries.

/// Shorthand fields that map to JQL clauses.
#[derive(Debug, Default)]
pub struct JqlShorthand {
    pub project: Option<String>,
    pub assignee: Option<String>,
    pub status: Option<String>,
    pub issue_type: Option<String>,
    pub priority: Option<String>,
    pub label: Option<String>,
}

impl JqlShorthand {
    /// Returns true if no shorthand fields are set.
    pub fn is_empty(&self) -> bool {
        self.project.is_none()
            && self.assignee.is_none()
            && self.status.is_none()
            && self.issue_type.is_none()
            && self.priority.is_none()
            && self.label.is_none()
    }

    /// Build a JQL query string from shorthand fields and optional raw JQL.
    ///
    /// Each non-None field becomes a clause joined with " AND ".
    /// Special handling: assignee "me" maps to `assignee = currentUser()`.
    /// Values containing double quotes are escaped.
    /// Returns None if no shorthand fields and no raw JQL are provided.
    pub fn build_jql(&self, raw_jql: Option<&str>) -> Option<String> {
        let mut clauses = Vec::new();

        if let Some(ref project) = self.project {
            clauses.push(format!("project = \"{}\"", escape_jql_value(project)));
        }
        if let Some(ref assignee) = self.assignee {
            if assignee.eq_ignore_ascii_case("me") {
                clauses.push("assignee = currentUser()".to_string());
            } else {
                clauses.push(format!("assignee = \"{}\"", escape_jql_value(assignee)));
            }
        }
        if let Some(ref status) = self.status {
            clauses.push(format!("status = \"{}\"", escape_jql_value(status)));
        }
        if let Some(ref issue_type) = self.issue_type {
            clauses.push(format!("issuetype = \"{}\"", escape_jql_value(issue_type)));
        }
        if let Some(ref priority) = self.priority {
            clauses.push(format!("priority = \"{}\"", escape_jql_value(priority)));
        }
        if let Some(ref label) = self.label {
            clauses.push(format!("labels = \"{}\"", escape_jql_value(label)));
        }

        // Append raw JQL if provided
        if let Some(raw) = raw_jql {
            let trimmed = raw.trim();
            if !trimmed.is_empty() {
                clauses.push(trimmed.to_string());
            }
        }

        if clauses.is_empty() {
            None
        } else {
            Some(clauses.join(" AND "))
        }
    }
}

/// Extract JQL shorthand flags from an arg list.
///
/// Scans for --jql, --project, --assignee, --status, --type, --priority, --label
/// and removes them (with their values) from the arg list.
/// Respects the `--` separator: stops extracting after encountering it.
///
/// Returns (JqlShorthand, Option<raw_jql>, remaining_args).
pub fn extract_jql_flags(args: &[String]) -> (JqlShorthand, Option<String>, Vec<String>) {
    let mut shorthand = JqlShorthand::default();
    let mut raw_jql: Option<String> = None;
    let mut remaining = Vec::new();
    let mut i = 0;
    let mut past_separator = false;

    while i < args.len() {
        let arg = &args[i];

        // Once we see --, everything after passes through unchanged
        if arg == "--" {
            past_separator = true;
            remaining.push(arg.clone());
            i += 1;
            continue;
        }

        if past_separator {
            remaining.push(arg.clone());
            i += 1;
            continue;
        }

        // Try --flag=value form
        if let Some((flag, value)) = arg.strip_prefix("--").and_then(|s| {
            let (f, v) = s.split_once('=')?;
            Some((f.to_string(), v.to_string()))
        }) {
            if assign_jql_field(&flag, &value, &mut shorthand, &mut raw_jql) {
                i += 1;
                continue;
            }
        }

        // Try --flag value form (space-separated)
        if let Some(flag) = arg.strip_prefix("--") {
            if is_jql_flag(flag) && i + 1 < args.len() {
                let value = args[i + 1].clone();
                assign_jql_field(flag, &value, &mut shorthand, &mut raw_jql);
                i += 2;
                continue;
            }
        }

        remaining.push(arg.clone());
        i += 1;
    }

    (shorthand, raw_jql, remaining)
}

/// Check if a flag name is a JQL shorthand flag.
fn is_jql_flag(flag: &str) -> bool {
    matches!(
        flag,
        "jql" | "project" | "assignee" | "status" | "type" | "priority" | "label"
    )
}

/// Assign a value to the appropriate JQL field. Returns true if the flag was recognised.
fn assign_jql_field(
    flag: &str,
    value: &str,
    shorthand: &mut JqlShorthand,
    raw_jql: &mut Option<String>,
) -> bool {
    match flag {
        "jql" => {
            *raw_jql = Some(value.to_string());
            true
        }
        "project" => {
            shorthand.project = Some(value.to_string());
            true
        }
        "assignee" => {
            shorthand.assignee = Some(value.to_string());
            true
        }
        "status" => {
            shorthand.status = Some(value.to_string());
            true
        }
        "type" => {
            shorthand.issue_type = Some(value.to_string());
            true
        }
        "priority" => {
            shorthand.priority = Some(value.to_string());
            true
        }
        "label" => {
            shorthand.label = Some(value.to_string());
            true
        }
        _ => false,
    }
}

/// Escape double quotes in a JQL value.
fn escape_jql_value(value: &str) -> String {
    value.replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_project_flag() {
        let s = JqlShorthand {
            project: Some("FOO".to_string()),
            ..Default::default()
        };
        let jql = s.build_jql(None).unwrap();
        assert_eq!(jql, r#"project = "FOO""#);
    }

    #[test]
    fn multiple_flags_joined_with_and() {
        let s = JqlShorthand {
            project: Some("FOO".to_string()),
            status: Some("Open".to_string()),
            ..Default::default()
        };
        let jql = s.build_jql(None).unwrap();
        assert_eq!(jql, r#"project = "FOO" AND status = "Open""#);
    }

    #[test]
    fn assignee_me_maps_to_current_user() {
        let s = JqlShorthand {
            assignee: Some("me".to_string()),
            ..Default::default()
        };
        let jql = s.build_jql(None).unwrap();
        assert_eq!(jql, "assignee = currentUser()");
    }

    #[test]
    fn assignee_me_case_insensitive() {
        let s = JqlShorthand {
            assignee: Some("ME".to_string()),
            ..Default::default()
        };
        let jql = s.build_jql(None).unwrap();
        assert_eq!(jql, "assignee = currentUser()");
    }

    #[test]
    fn combining_shorthand_with_raw_jql() {
        let s = JqlShorthand {
            project: Some("FOO".to_string()),
            ..Default::default()
        };
        let jql = s.build_jql(Some("priority = High")).unwrap();
        assert_eq!(jql, r#"project = "FOO" AND priority = High"#);
    }

    #[test]
    fn empty_shorthand_no_raw_returns_none() {
        let s = JqlShorthand::default();
        assert!(s.build_jql(None).is_none());
    }

    #[test]
    fn raw_jql_only() {
        let s = JqlShorthand::default();
        let jql = s.build_jql(Some("project = BAR")).unwrap();
        assert_eq!(jql, "project = BAR");
    }

    #[test]
    fn status_with_spaces_is_quoted() {
        let s = JqlShorthand {
            status: Some("In Progress".to_string()),
            ..Default::default()
        };
        let jql = s.build_jql(None).unwrap();
        assert_eq!(jql, r#"status = "In Progress""#);
    }

    #[test]
    fn value_with_embedded_double_quotes_is_escaped() {
        let s = JqlShorthand {
            status: Some(r#"Done "Final""#.to_string()),
            ..Default::default()
        };
        let jql = s.build_jql(None).unwrap();
        assert_eq!(jql, r#"status = "Done \"Final\"""#);
    }

    #[test]
    fn is_empty_when_all_none() {
        let s = JqlShorthand::default();
        assert!(s.is_empty());
    }

    #[test]
    fn is_not_empty_when_any_set() {
        let s = JqlShorthand {
            label: Some("bug".to_string()),
            ..Default::default()
        };
        assert!(!s.is_empty());
    }

    // --- extract_jql_flags tests ---

    fn s(vals: &[&str]) -> Vec<String> {
        vals.iter().map(|v| v.to_string()).collect()
    }

    #[test]
    fn extract_single_flag() {
        let (sh, jql, remaining) = extract_jql_flags(&s(&["--project", "KAN"]));
        assert_eq!(sh.project.as_deref(), Some("KAN"));
        assert!(jql.is_none());
        assert!(remaining.is_empty());
    }

    #[test]
    fn extract_multiple_flags() {
        let (sh, jql, remaining) =
            extract_jql_flags(&s(&["--project", "KAN", "--status", "Open", "--assignee", "me"]));
        assert_eq!(sh.project.as_deref(), Some("KAN"));
        assert_eq!(sh.status.as_deref(), Some("Open"));
        assert_eq!(sh.assignee.as_deref(), Some("me"));
        assert!(jql.is_none());
        assert!(remaining.is_empty());
    }

    #[test]
    fn extract_equals_style() {
        let (sh, jql, remaining) =
            extract_jql_flags(&s(&["--project=KAN", "--jql=priority = High"]));
        assert_eq!(sh.project.as_deref(), Some("KAN"));
        assert_eq!(jql.as_deref(), Some("priority = High"));
        assert!(remaining.is_empty());
    }

    #[test]
    fn extract_mixed_with_non_jql_flags() {
        let (sh, _jql, remaining) = extract_jql_flags(&s(&[
            "--expand", "names", "--project", "KAN", "--fields", "summary",
        ]));
        assert_eq!(sh.project.as_deref(), Some("KAN"));
        assert_eq!(remaining, s(&["--expand", "names", "--fields", "summary"]));
    }

    #[test]
    fn extract_no_jql_flags() {
        let (sh, jql, remaining) = extract_jql_flags(&s(&["--expand", "names", "--limit", "50"]));
        assert!(sh.is_empty());
        assert!(jql.is_none());
        assert_eq!(remaining, s(&["--expand", "names", "--limit", "50"]));
    }

    #[test]
    fn extract_type_maps_to_issue_type() {
        let (sh, _, _) = extract_jql_flags(&s(&["--type", "Bug"]));
        assert_eq!(sh.issue_type.as_deref(), Some("Bug"));
    }

    #[test]
    fn extract_raw_jql() {
        let (sh, jql, remaining) = extract_jql_flags(&s(&["--jql", "project = BAR"]));
        assert!(sh.is_empty());
        assert_eq!(jql.as_deref(), Some("project = BAR"));
        assert!(remaining.is_empty());
    }

    #[test]
    fn extract_stops_at_separator() {
        let (sh, _, remaining) =
            extract_jql_flags(&s(&["--project", "KAN", "--", "--status", "Open"]));
        assert_eq!(sh.project.as_deref(), Some("KAN"));
        assert!(sh.status.is_none(), "--status after -- should not be extracted");
        assert_eq!(remaining, s(&["--", "--status", "Open"]));
    }

    // --- existing build_jql tests ---

    #[test]
    fn all_fields_combined() {
        let s = JqlShorthand {
            project: Some("PROJ".to_string()),
            assignee: Some("me".to_string()),
            status: Some("Open".to_string()),
            issue_type: Some("Bug".to_string()),
            priority: Some("High".to_string()),
            label: Some("urgent".to_string()),
        };
        let jql = s.build_jql(None).unwrap();
        assert!(jql.contains("project = \"PROJ\""));
        assert!(jql.contains("assignee = currentUser()"));
        assert!(jql.contains("status = \"Open\""));
        assert!(jql.contains("issuetype = \"Bug\""));
        assert!(jql.contains("priority = \"High\""));
        assert!(jql.contains("labels = \"urgent\""));
        // All joined by AND
        assert_eq!(jql.matches(" AND ").count(), 5);
    }
}
