//! Browser launch helper for --web flag.
//!
//! Constructs Atlassian Cloud URLs for known entity types and opens them
//! in the user's default browser via the `open` crate.

use crate::core::error::ShrugError;

/// Open a URL in the default browser.
pub fn open_in_browser(url: &str) -> Result<(), ShrugError> {
    tracing::debug!(url = %url, "Opening in browser");
    open::that(url).map_err(|e| {
        ShrugError::UsageError(format!("Failed to open browser: {}", e))
    })
}

/// Build a Jira issue URL for --web.
pub fn jira_issue_url(site: &str, key: &str) -> String {
    format!("{}/browse/{}", site.trim_end_matches('/'), key)
}

/// Build a Jira project URL for --web.
pub fn jira_project_url(site: &str, key: &str) -> String {
    format!("{}/browse/{}", site.trim_end_matches('/'), key)
}

/// Build a Jira dashboard URL for --web.
pub fn jira_dashboard_url(site: &str, id: &str) -> String {
    format!(
        "{}/jira/dashboards/{}",
        site.trim_end_matches('/'),
        id
    )
}

/// Build a Jira filter URL for --web.
pub fn jira_filter_url(site: &str, id: &str) -> String {
    format!(
        "{}/issues/?filter={}",
        site.trim_end_matches('/'),
        id
    )
}

/// Build a Jira board URL for --web.
pub fn jira_board_url(site: &str, id: &str) -> String {
    format!(
        "{}/jira/software/projects?board={}",
        site.trim_end_matches('/'),
        id
    )
}

/// Build a Jira sprint URL for --web (board context needed).
pub fn jira_sprint_url(site: &str, id: &str) -> String {
    format!(
        "{}/jira/software/projects?sprint={}",
        site.trim_end_matches('/'),
        id
    )
}

/// Build a Confluence page URL for --web.
pub fn confluence_page_url(site: &str, id: &str) -> String {
    format!(
        "{}/wiki/pages/viewpage.action?pageId={}",
        site.trim_end_matches('/'),
        id
    )
}

/// Build a Confluence space URL for --web.
pub fn confluence_space_url(site: &str, key: &str) -> String {
    format!(
        "{}/wiki/spaces/{}",
        site.trim_end_matches('/'),
        key
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jira_issue_url() {
        let url = jira_issue_url("https://site.atlassian.net", "TEAM-123");
        assert_eq!(url, "https://site.atlassian.net/browse/TEAM-123");
    }

    #[test]
    fn test_jira_issue_url_trailing_slash() {
        let url = jira_issue_url("https://site.atlassian.net/", "TEAM-1");
        assert_eq!(url, "https://site.atlassian.net/browse/TEAM-1");
    }

    #[test]
    fn test_confluence_page_url() {
        let url = confluence_page_url("https://site.atlassian.net", "12345");
        assert_eq!(
            url,
            "https://site.atlassian.net/wiki/pages/viewpage.action?pageId=12345"
        );
    }

    #[test]
    fn test_confluence_space_url() {
        let url = confluence_space_url("https://site.atlassian.net", "DOCS");
        assert_eq!(url, "https://site.atlassian.net/wiki/spaces/DOCS");
    }

    #[test]
    fn test_jira_dashboard_url() {
        let url = jira_dashboard_url("https://site.atlassian.net", "10001");
        assert_eq!(url, "https://site.atlassian.net/jira/dashboards/10001");
    }
}
