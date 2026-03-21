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
