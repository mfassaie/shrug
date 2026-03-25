//! Jira issue entity: LCRUD operations for Jira Cloud issues.
//!
//! First entity implementation (proof of concept). All subsequent entities
//! follow the same pattern: Commands enum, body builders, execute function.

pub mod attachment;
pub mod comment;
pub mod link;
pub mod property;
pub mod remote_link;
pub mod watcher;
pub mod worklog;

use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};

use clap::Subcommand;
use reqwest::blocking::Client;
use reqwest::Method;
use serde_json::{json, Value};

use crate::auth::credentials::ResolvedCredential;
use crate::cli::{ColorChoice, OutputFormat};
use crate::content::jql::JqlShorthand;
use crate::content::markdown_to_adf;
use crate::core::error::ShrugError;
use crate::core::http;
use crate::core::output;

/// Issue entity subcommands.
#[derive(Subcommand)]
pub enum IssueCommands {
    /// List issues using JQL search
    List {
        /// Raw JQL query (overrides all shorthand flags)
        #[arg(long)]
        jql: Option<String>,
        /// Filter by project key
        #[arg(long)]
        project: Option<String>,
        /// Filter by assignee (use @me for current user)
        #[arg(short = 'a', long)]
        assignee: Option<String>,
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        /// Filter by issue type
        #[arg(long = "type")]
        issue_type: Option<String>,
        /// Filter by priority
        #[arg(long)]
        priority: Option<String>,
        /// Filter by label
        #[arg(short = 'l', long)]
        label: Option<String>,
        /// Filter by last updated (e.g., -7d, -1w)
        #[arg(long)]
        updated: Option<String>,
        /// JQL ORDER BY clause (e.g., "priority DESC")
        #[arg(long)]
        order_by: Option<String>,
        /// Comma-separated list of fields to return
        #[arg(long, value_delimiter = ',')]
        fields: Option<Vec<String>>,
    },
    /// Create a new issue
    Create {
        /// Issue summary (title)
        #[arg(short = 's', long)]
        summary: String,
        /// Project key (e.g., TEAM)
        #[arg(long)]
        project: String,
        /// Issue type (e.g., Bug, Task, Story)
        #[arg(long = "type")]
        issue_type: String,
        /// Description in markdown (converted to ADF)
        #[arg(short = 'b', long, conflicts_with = "body_file")]
        body: Option<String>,
        /// Read description from file (use - for stdin)
        #[arg(long, conflicts_with = "body")]
        body_file: Option<String>,
        /// Assignee (accountId or @me)
        #[arg(short = 'a', long)]
        assignee: Option<String>,
        /// Reporter (accountId or @me)
        #[arg(long)]
        reporter: Option<String>,
        /// Priority name (e.g., High, Medium)
        #[arg(long)]
        priority: Option<String>,
        /// Labels (repeatable)
        #[arg(short = 'l', long)]
        label: Vec<String>,
        /// Components (repeatable)
        #[arg(long)]
        component: Vec<String>,
        /// Fix versions (repeatable)
        #[arg(long)]
        fix_version: Vec<String>,
        /// Parent issue key (for subtasks)
        #[arg(long)]
        parent: Option<String>,
        /// Due date (YYYY-MM-DD)
        #[arg(long)]
        due_date: Option<String>,
        /// Full JSON payload from file (overrides all typed flags)
        #[arg(long)]
        from_json: Option<String>,
        /// Arbitrary field (key=value, repeatable)
        #[arg(long)]
        field: Vec<String>,
    },
    /// View an issue
    View {
        /// Issue key (e.g., TEAM-123)
        key: String,
    },
    /// Edit an issue
    Edit {
        /// Issue key (e.g., TEAM-123)
        key: String,
        /// New summary
        #[arg(short = 's', long)]
        summary: Option<String>,
        /// New description in markdown
        #[arg(short = 'b', long, conflicts_with = "body_file")]
        body: Option<String>,
        /// Read description from file (use - for stdin)
        #[arg(long, conflicts_with = "body")]
        body_file: Option<String>,
        /// Change assignee (accountId, @me, or "none" to unassign)
        #[arg(short = 'a', long)]
        assignee: Option<String>,
        /// Change reporter
        #[arg(long)]
        reporter: Option<String>,
        /// Change priority
        #[arg(long)]
        priority: Option<String>,
        /// Change parent (or "none" to remove)
        #[arg(long)]
        parent: Option<String>,
        /// Change due date (YYYY-MM-DD)
        #[arg(long)]
        due_date: Option<String>,
        /// Add label (repeatable)
        #[arg(long)]
        add_label: Vec<String>,
        /// Remove label (repeatable)
        #[arg(long)]
        remove_label: Vec<String>,
        /// Add component (repeatable)
        #[arg(long)]
        add_component: Vec<String>,
        /// Remove component (repeatable)
        #[arg(long)]
        remove_component: Vec<String>,
        /// Add fix version (repeatable)
        #[arg(long)]
        add_fix_version: Vec<String>,
        /// Remove fix version (repeatable)
        #[arg(long)]
        remove_fix_version: Vec<String>,
        /// Full JSON payload from file (overrides all typed flags)
        #[arg(long)]
        from_json: Option<String>,
    },
    /// Delete an issue
    Delete {
        /// Issue key (e.g., TEAM-123)
        key: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Comment operations on an issue
    Comment {
        #[command(subcommand)]
        command: comment::CommentCommands,
    },
    /// Worklog operations on an issue
    Worklog {
        #[command(subcommand)]
        command: worklog::WorklogCommands,
    },
    /// Attachment operations on an issue
    Attachment {
        #[command(subcommand)]
        command: attachment::AttachmentCommands,
    },
    /// Watcher operations on an issue
    Watcher {
        #[command(subcommand)]
        command: watcher::WatcherCommands,
    },
    /// Issue link operations
    Link {
        #[command(subcommand)]
        command: link::LinkCommands,
    },
    /// Remote link operations on an issue
    #[command(name = "remote-link")]
    RemoteLink {
        #[command(subcommand)]
        command: remote_link::RemoteLinkCommands,
    },
    /// Issue property operations
    Property {
        #[command(subcommand)]
        command: property::PropertyCommands,
    },
}

/// Execute an issue command.
pub fn execute(
    cmd: &IssueCommands,
    credential: &ResolvedCredential,
    client: &Client,
    output_format: &OutputFormat,
    color: &ColorChoice,
    limit: Option<u32>,
    dry_run: bool,
) -> Result<(), ShrugError> {
    let base_url = http::build_base_url(credential);
    let color_enabled = match color {
        ColorChoice::Always => true,
        ColorChoice::Never => false,
        ColorChoice::Auto => {
            std::env::var("NO_COLOR").is_err()
                && is_terminal::is_terminal(std::io::stdout())
        }
    };

    match cmd {
        IssueCommands::List {
            jql,
            project,
            assignee,
            status,
            issue_type,
            priority,
            label,
            updated,
            order_by,
            fields,
        } => {
            let jql_query = build_list_jql(
                jql.as_deref(),
                project.as_deref(),
                assignee.as_deref(),
                status.as_deref(),
                issue_type.as_deref(),
                priority.as_deref(),
                label.as_deref(),
                updated.as_deref(),
                order_by.as_deref(),
            );

            let base_search_url = format!("{}/rest/api/3/search/jql", base_url);
            let mut query_params: Vec<(String, String)> = Vec::new();
            // /search/jql requires a bounded JQL query — default to recent issues
            let jql = jql_query.unwrap_or_else(|| "created >= -30d ORDER BY created DESC".to_string());
            query_params.push(("jql".to_string(), jql));
            if let Some(ref f) = fields {
                query_params.push(("fields".to_string(), f.join(",")));
            }

            if dry_run {
                let url = http::build_url(&base_search_url, "", &HashMap::new(), &query_params);
                http::dry_run_request(&Method::GET, &url, None);
                return Ok(());
            }

            // GET-based pagination (new /search/jql endpoint)
            let page_size: u32 = 50;
            let effective_limit = limit.unwrap_or(u32::MAX) as usize;
            let mut all_issues: Vec<serde_json::Value> = Vec::new();
            let mut start_at: u64 = 0;

            loop {
                let mut page_params = query_params.clone();
                page_params.push(("startAt".to_string(), start_at.to_string()));
                page_params.push(("maxResults".to_string(), page_size.to_string()));

                let url = http::build_url(&base_search_url, "", &HashMap::new(), &page_params);
                let result = http::execute_request(
                    client, Method::GET, &url, Some(credential),
                    None, &[],
                )?;

                let json_val = match result {
                    Some(v) => v,
                    None => break,
                };

                let page_issues = crate::core::pagination::extract_results(&json_val)
                    .cloned().unwrap_or_default();
                let count = page_issues.len() as u32;

                if count == 0 { break; }

                all_issues.extend(page_issues);

                if all_issues.len() >= effective_limit {
                    all_issues.truncate(effective_limit);
                    break;
                }

                if !crate::core::pagination::has_more_offset(&json_val, start_at, count) {
                    break;
                }

                start_at += count as u64;
            }

            let json_val = serde_json::Value::Array(all_issues);
            if !json_val.as_array().is_none_or(|a| a.is_empty()) {
                let formatted = output::format_response(
                    &json_val.to_string(), output_format,
                    is_terminal::is_terminal(std::io::stdout()), color_enabled,
                    fields.as_deref(),
                );
                println!("{}", formatted);
            }
            Ok(())
        }

        IssueCommands::Create {
            summary,
            project,
            issue_type,
            body,
            body_file,
            assignee,
            reporter,
            priority,
            label,
            component,
            fix_version,
            parent,
            due_date,
            from_json,
            field,
        } => {
            let request_body = if let Some(ref path) = from_json {
                tracing::debug!("Using --from-json, ignoring typed flags");
                read_json_file(path)?
            } else {
                let description = read_description(body.as_deref(), body_file.as_deref())?;
                let resolved_assignee =
                    resolve_at_me(assignee.as_deref(), client, credential, &base_url)?;
                let resolved_reporter =
                    resolve_at_me(reporter.as_deref(), client, credential, &base_url)?;
                build_create_body(
                    summary,
                    project,
                    issue_type,
                    description.as_ref(),
                    resolved_assignee.as_deref(),
                    resolved_reporter.as_deref(),
                    priority.as_deref(),
                    label,
                    component,
                    fix_version,
                    parent.as_deref(),
                    due_date.as_deref(),
                    field,
                )
            };

            let url = format!("{}/rest/api/3/issue", base_url);

            if dry_run {
                http::dry_run_request(&Method::POST, &url, Some(&request_body));
                return Ok(());
            }

            let result = http::execute_request(
                client,
                Method::POST,
                &url,
                Some(credential),
                Some(&request_body),
                &[],
            )?;

            if let Some(ref json_val) = result {
                match output_format {
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(json_val).unwrap_or_default()
                        );
                    }
                    _ => {
                        if let Some(key) = json_val.get("key").and_then(|v| v.as_str()) {
                            println!("Created {}", key);
                        }
                    }
                }
            }
            Ok(())
        }

        IssueCommands::View { key } => {
            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), key.clone());
            let url = http::build_url(
                &base_url,
                "/rest/api/3/issue/{issueIdOrKey}",
                &path_params,
                &[],
            );

            if dry_run {
                http::dry_run_request(&Method::GET, &url, None);
                return Ok(());
            }

            let result = http::execute_request(
                client,
                Method::GET,
                &url,
                Some(credential),
                None,
                &[],
            )?;

            if let Some(ref json_val) = result {
                let formatted = output::format_response(
                    &json_val.to_string(),
                    output_format,
                    is_terminal::is_terminal(std::io::stdout()),
                    color_enabled,
                    None,
                );
                println!("{}", formatted);
            }
            Ok(())
        }

        IssueCommands::Edit {
            key,
            summary,
            body,
            body_file,
            assignee,
            reporter,
            priority,
            parent,
            due_date,
            add_label,
            remove_label,
            add_component,
            remove_component,
            add_fix_version,
            remove_fix_version,
            from_json,
        } => {
            let request_body = if let Some(ref path) = from_json {
                tracing::debug!("Using --from-json, ignoring typed flags");
                read_json_file(path)?
            } else {
                let description = read_description(body.as_deref(), body_file.as_deref())?;
                let resolved_assignee =
                    resolve_at_me(assignee.as_deref(), client, credential, &base_url)?;
                let resolved_reporter =
                    resolve_at_me(reporter.as_deref(), client, credential, &base_url)?;
                build_edit_body(
                    summary.as_deref(),
                    description.as_ref(),
                    resolved_assignee.as_deref(),
                    resolved_reporter.as_deref(),
                    priority.as_deref(),
                    parent.as_deref(),
                    due_date.as_deref(),
                    add_label,
                    remove_label,
                    add_component,
                    remove_component,
                    add_fix_version,
                    remove_fix_version,
                )
            };

            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), key.clone());
            let url = http::build_url(
                &base_url,
                "/rest/api/3/issue/{issueIdOrKey}",
                &path_params,
                &[],
            );

            if dry_run {
                http::dry_run_request(&Method::PUT, &url, Some(&request_body));
                return Ok(());
            }

            http::execute_request(
                client,
                Method::PUT,
                &url,
                Some(credential),
                Some(&request_body),
                &[],
            )?;

            match output_format {
                OutputFormat::Json => {
                    println!("{}", json!({"key": key, "status": "updated"}));
                }
                _ => println!("Updated {}", key),
            }
            Ok(())
        }

        IssueCommands::Delete { key, yes } => {
            let mut path_params = HashMap::new();
            path_params.insert("issueIdOrKey".to_string(), key.clone());
            let url = http::build_url(
                &base_url,
                "/rest/api/3/issue/{issueIdOrKey}",
                &path_params,
                &[],
            );

            if dry_run {
                http::dry_run_request(&Method::DELETE, &url, None);
                return Ok(());
            }

            if !yes && !confirm_delete(key)? {
                return Ok(());
            }

            http::execute_request(
                client,
                Method::DELETE,
                &url,
                Some(credential),
                None,
                &[],
            )?;

            match output_format {
                OutputFormat::Json => {
                    println!("{}", json!({"key": key, "status": "deleted"}));
                }
                _ => println!("Deleted {}", key),
            }
            Ok(())
        }

        IssueCommands::Comment { command } => {
            comment::execute(command, credential, client, &base_url, output_format, color_enabled)
        }
        IssueCommands::Worklog { command } => {
            worklog::execute(command, credential, client, &base_url, output_format, color_enabled)
        }
        IssueCommands::Attachment { command } => {
            attachment::execute(command, credential, client, &base_url, output_format, color_enabled)
        }
        IssueCommands::Watcher { command } => {
            watcher::execute(command, credential, client, &base_url, output_format, color_enabled)
        }
        IssueCommands::Link { command } => {
            link::execute(command, credential, client, &base_url, output_format, color_enabled)
        }
        IssueCommands::RemoteLink { command } => {
            remote_link::execute(command, credential, client, &base_url, output_format, color_enabled)
        }
        IssueCommands::Property { command } => {
            property::execute(command, credential, client, &base_url, output_format, color_enabled)
        }
    }
}

// ---------------------------------------------------------------------------
// Body builders
// ---------------------------------------------------------------------------

/// Build JQL query string from typed flags.
///
/// If `--jql` is provided, it overrides all shorthand flags, `--updated`, and `--order-by`.
#[allow(clippy::too_many_arguments)]
pub fn build_list_jql(
    jql: Option<&str>,
    project: Option<&str>,
    assignee: Option<&str>,
    status: Option<&str>,
    issue_type: Option<&str>,
    priority: Option<&str>,
    label: Option<&str>,
    updated: Option<&str>,
    order_by: Option<&str>,
) -> Option<String> {
    if let Some(raw) = jql {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    // Normalise @me → "me" for JqlShorthand (which maps "me" → currentUser())
    let assignee_normalised = assignee.map(|a| {
        if a.eq_ignore_ascii_case("@me") {
            "me".to_string()
        } else {
            a.to_string()
        }
    });

    let shorthand = JqlShorthand {
        project: project.map(String::from),
        assignee: assignee_normalised,
        status: status.map(String::from),
        issue_type: issue_type.map(String::from),
        priority: priority.map(String::from),
        label: label.map(String::from),
    };

    let mut result = shorthand.build_jql(None);

    if let Some(updated_val) = updated {
        let clause = format!("updated >= \"{}\"", updated_val);
        result = Some(match result {
            Some(existing) => format!("{} AND {}", existing, clause),
            None => clause,
        });
    }

    if let Some(order) = order_by {
        result = Some(match result {
            Some(existing) => format!("{} ORDER BY {}", existing, order),
            None => format!("ORDER BY {}", order),
        });
    }

    result
}

/// Build JSON request body for issue creation.
#[allow(clippy::too_many_arguments)]
pub fn build_create_body(
    summary: &str,
    project: &str,
    issue_type: &str,
    description: Option<&Value>,
    assignee: Option<&str>,
    reporter: Option<&str>,
    priority: Option<&str>,
    labels: &[String],
    components: &[String],
    fix_versions: &[String],
    parent: Option<&str>,
    due_date: Option<&str>,
    extra_fields: &[String],
) -> Value {
    let mut fields = json!({
        "summary": summary,
        "project": {"key": project},
        "issuetype": {"name": issue_type},
    });

    if let Some(desc) = description {
        fields["description"] = desc.clone();
    }
    if let Some(a) = assignee {
        fields["assignee"] = json!({"accountId": a});
    }
    if let Some(r) = reporter {
        fields["reporter"] = json!({"accountId": r});
    }
    if let Some(p) = priority {
        fields["priority"] = json!({"name": p});
    }
    if !labels.is_empty() {
        fields["labels"] = json!(labels);
    }
    if !components.is_empty() {
        fields["components"] = Value::Array(
            components.iter().map(|c| json!({"name": c})).collect(),
        );
    }
    if !fix_versions.is_empty() {
        fields["fixVersions"] = Value::Array(
            fix_versions.iter().map(|v| json!({"name": v})).collect(),
        );
    }
    if let Some(p) = parent {
        fields["parent"] = json!({"key": p});
    }
    if let Some(d) = due_date {
        fields["duedate"] = json!(d);
    }

    for kv in extra_fields {
        if let Some((key, val)) = kv.split_once('=') {
            let key = key.trim();
            let val = val.trim();
            let json_val: Value = serde_json::from_str(val).unwrap_or_else(|_| json!(val));
            fields[key] = json_val;
        }
    }

    json!({"fields": fields})
}

/// Build JSON request body for issue edit.
///
/// Uses "fields" for direct replacements and "update" for array operations
/// (add/remove on labels, components, fix versions).
#[allow(clippy::too_many_arguments)]
pub fn build_edit_body(
    summary: Option<&str>,
    description: Option<&Value>,
    assignee: Option<&str>,
    reporter: Option<&str>,
    priority: Option<&str>,
    parent: Option<&str>,
    due_date: Option<&str>,
    add_labels: &[String],
    remove_labels: &[String],
    add_components: &[String],
    remove_components: &[String],
    add_fix_versions: &[String],
    remove_fix_versions: &[String],
) -> Value {
    let mut result = json!({});
    let mut fields = json!({});
    let mut update = json!({});

    if let Some(s) = summary {
        fields["summary"] = json!(s);
    }
    if let Some(desc) = description {
        fields["description"] = desc.clone();
    }
    if let Some(a) = assignee {
        if a.eq_ignore_ascii_case("none") {
            fields["assignee"] = Value::Null;
        } else {
            fields["assignee"] = json!({"accountId": a});
        }
    }
    if let Some(r) = reporter {
        fields["reporter"] = json!({"accountId": r});
    }
    if let Some(p) = priority {
        fields["priority"] = json!({"name": p});
    }
    if let Some(p) = parent {
        if p.eq_ignore_ascii_case("none") {
            fields["parent"] = Value::Null;
        } else {
            fields["parent"] = json!({"key": p});
        }
    }
    if let Some(d) = due_date {
        fields["duedate"] = json!(d);
    }

    // Label add/remove operations
    let mut label_ops: Vec<Value> = Vec::new();
    for l in add_labels {
        label_ops.push(json!({"add": l}));
    }
    for l in remove_labels {
        label_ops.push(json!({"remove": l}));
    }
    if !label_ops.is_empty() {
        update["labels"] = Value::Array(label_ops);
    }

    // Component add/remove operations
    let mut comp_ops: Vec<Value> = Vec::new();
    for c in add_components {
        comp_ops.push(json!({"add": {"name": c}}));
    }
    for c in remove_components {
        comp_ops.push(json!({"remove": {"name": c}}));
    }
    if !comp_ops.is_empty() {
        update["components"] = Value::Array(comp_ops);
    }

    // Fix version add/remove operations
    let mut ver_ops: Vec<Value> = Vec::new();
    for v in add_fix_versions {
        ver_ops.push(json!({"add": {"name": v}}));
    }
    for v in remove_fix_versions {
        ver_ops.push(json!({"remove": {"name": v}}));
    }
    if !ver_ops.is_empty() {
        update["fixVersions"] = Value::Array(ver_ops);
    }

    if fields != json!({}) {
        result["fields"] = fields;
    }
    if update != json!({}) {
        result["update"] = update;
    }

    result
}

// ---------------------------------------------------------------------------
// Input helpers
// ---------------------------------------------------------------------------

/// Read description from `--body` or `--body-file`, converting markdown to ADF.
pub fn read_description(
    body: Option<&str>,
    body_file: Option<&str>,
) -> Result<Option<Value>, ShrugError> {
    if let Some(text) = body {
        return Ok(Some(markdown_to_adf::markdown_to_adf(text)));
    }
    if let Some(path) = body_file {
        let content = if path == "-" {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).map_err(|e| {
                ShrugError::UsageError(format!("Failed to read from stdin: {}", e))
            })?;
            buf
        } else {
            fs::read_to_string(path).map_err(|e| {
                ShrugError::UsageError(format!("Failed to read {}: {}", path, e))
            })?
        };
        return Ok(Some(markdown_to_adf::markdown_to_adf(&content)));
    }
    Ok(None)
}

/// Read a JSON file for `--from-json`. Supports `-` for stdin.
pub fn read_json_file(path: &str) -> Result<Value, ShrugError> {
    let content = if path == "-" {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).map_err(|e| {
            ShrugError::UsageError(format!("Failed to read from stdin: {}", e))
        })?;
        buf
    } else {
        fs::read_to_string(path).map_err(|e| {
            ShrugError::UsageError(format!("Failed to read {}: {}", path, e))
        })?
    };
    serde_json::from_str(&content).map_err(|e| {
        ShrugError::UsageError(format!("Invalid JSON in {}: {}", path, e))
    })
}

// ---------------------------------------------------------------------------
// Auth helpers
// ---------------------------------------------------------------------------

/// Resolve `@me` to the current user's accountId via GET /rest/api/3/myself.
fn resolve_myself(
    client: &Client,
    credential: &ResolvedCredential,
    base_url: &str,
) -> Result<String, ShrugError> {
    let url = format!("{}/rest/api/3/myself", base_url);
    let result = http::execute_request(client, Method::GET, &url, Some(credential), None, &[])?;
    match result {
        Some(json_val) => json_val
            .get("accountId")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| {
                ShrugError::UsageError(
                    "Could not resolve @me: accountId not in response. Use an accountId directly."
                        .into(),
                )
            }),
        None => Err(ShrugError::UsageError(
            "Could not resolve @me: empty response from /myself. Use an accountId directly.".into(),
        )),
    }
}

/// If the value is `@me` (case-insensitive), resolve via the API. Otherwise pass through.
pub fn resolve_at_me(
    value: Option<&str>,
    client: &Client,
    credential: &ResolvedCredential,
    base_url: &str,
) -> Result<Option<String>, ShrugError> {
    match value {
        Some(v) if v.eq_ignore_ascii_case("@me") => {
            Ok(Some(resolve_myself(client, credential, base_url)?))
        }
        Some(v) => Ok(Some(v.to_string())),
        None => Ok(None),
    }
}

// ---------------------------------------------------------------------------
// Confirmation
// ---------------------------------------------------------------------------

/// Prompt for delete confirmation. Returns `true` if the user confirms.
///
/// Errors if stderr is not a TTY and `--yes` was not provided.
fn confirm_delete(key: &str) -> Result<bool, ShrugError> {
    confirm_delete_prompt(&format!("Delete issue {}? (y/N): ", key))
}

/// Prompt for delete confirmation with a custom message. Returns `true` if the user confirms.
///
/// Errors if stderr is not a TTY and `--yes` was not provided.
pub fn confirm_delete_prompt(prompt: &str) -> Result<bool, ShrugError> {
    if !is_terminal::is_terminal(std::io::stderr()) {
        return Err(ShrugError::UsageError(
            "Use --yes to confirm deletion in non-interactive mode".into(),
        ));
    }
    eprint!("{}", prompt);
    io::stderr().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| {
        ShrugError::UsageError(format!("Failed to read confirmation: {}", e))
    })?;
    Ok(input.trim().eq_ignore_ascii_case("y"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;

    #[test]
    fn test_issue_list_jql_from_shorthand() {
        let jql = build_list_jql(
            None,
            Some("TEAM"),
            Some("@me"),
            Some("Open"),
            None,
            None,
            None,
            None,
            None,
        );
        let q = jql.unwrap();
        assert!(q.contains("project = \"TEAM\""));
        assert!(q.contains("assignee = currentUser()"));
        assert!(q.contains("status = \"Open\""));
    }

    #[test]
    fn test_issue_create_with_params() {
        let body = build_create_body(
            "Fix bug",
            "TEAM",
            "Bug",
            None,
            None,
            None,
            Some("High"),
            &["backend".into(), "urgent".into()],
            &[],
            &[],
            None,
            None,
            &[],
        );
        let fields = &body["fields"];
        assert_eq!(fields["summary"], "Fix bug");
        assert_eq!(fields["project"]["key"], "TEAM");
        assert_eq!(fields["issuetype"]["name"], "Bug");
        assert_eq!(fields["priority"]["name"], "High");
        assert_eq!(fields["labels"], json!(["backend", "urgent"]));
    }

    #[test]
    fn test_issue_create_with_from_json() {
        // --from-json provides the entire body; it should NOT be wrapped in {"fields": ...}
        let raw = json!({"fields": {"summary": "From file", "project": {"id": "10000"}}});
        assert_eq!(raw["fields"]["summary"], "From file");
        assert_eq!(raw["fields"]["project"]["id"], "10000");
    }

    #[test]
    fn test_issue_view_url() {
        let mut path_params = HashMap::new();
        path_params.insert("issueIdOrKey".to_string(), "TEAM-123".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/issue/{issueIdOrKey}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/api/3/issue/TEAM-123"
        );
    }

    #[test]
    fn test_issue_edit_with_params() {
        let body = build_edit_body(
            Some("New title"),
            None,
            None,
            None,
            Some("Critical"),
            None,
            None,
            &["urgent".into()],
            &["stale".into()],
            &[],
            &[],
            &[],
            &[],
        );
        assert_eq!(body["fields"]["summary"], "New title");
        assert_eq!(body["fields"]["priority"]["name"], "Critical");
        let labels = body["update"]["labels"].as_array().unwrap();
        assert_eq!(labels[0], json!({"add": "urgent"}));
        assert_eq!(labels[1], json!({"remove": "stale"}));
    }

    #[test]
    fn test_issue_edit_with_from_json() {
        let custom = json!({"fields": {"summary": "From JSON"}});
        // --from-json passes through unchanged
        assert_eq!(custom["fields"]["summary"], "From JSON");
    }

    #[test]
    fn test_issue_delete_url() {
        let mut path_params = HashMap::new();
        path_params.insert("issueIdOrKey".to_string(), "TEAM-456".to_string());
        let url = http::build_url(
            "https://site.atlassian.net",
            "/rest/api/3/issue/{issueIdOrKey}",
            &path_params,
            &[],
        );
        assert_eq!(
            url,
            "https://site.atlassian.net/rest/api/3/issue/TEAM-456"
        );
    }

    #[test]
    fn test_issue_list_raw_jql_overrides_shorthand() {
        let jql = build_list_jql(
            Some("project = CUSTOM ORDER BY created"),
            Some("TEAM"),
            Some("@me"),
            Some("Open"),
            None,
            None,
            None,
            Some("-7d"),
            Some("priority"),
        );
        assert_eq!(jql.unwrap(), "project = CUSTOM ORDER BY created");
    }

    #[test]
    fn test_issue_list_updated_flag() {
        let jql = build_list_jql(
            None,
            Some("TEAM"),
            None,
            None,
            None,
            None,
            None,
            Some("-7d"),
            None,
        );
        let q = jql.unwrap();
        assert!(q.contains("project = \"TEAM\""));
        assert!(q.contains("updated >= \"-7d\""));
    }

    #[test]
    fn test_issue_list_order_by() {
        let jql = build_list_jql(
            None,
            Some("TEAM"),
            None,
            None,
            None,
            None,
            None,
            None,
            Some("priority DESC"),
        );
        let q = jql.unwrap();
        assert!(q.starts_with("project = \"TEAM\""));
        assert!(q.ends_with("ORDER BY priority DESC"));
        // ORDER BY is not joined with AND
        assert!(!q.contains("AND ORDER BY"));
    }

    #[test]
    fn test_issue_create_from_json_ignores_typed_flags() {
        // When --from-json is set, execute() uses its content directly.
        // build_create_body is skipped entirely. This test verifies the
        // read_json_file path returns raw content without wrapping.
        use std::io::Write as _;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("input.json");
        let mut file = fs::File::create(&path).unwrap();
        write!(file, r#"{{"custom": "payload"}}"#).unwrap();

        let value = read_json_file(path.to_str().unwrap()).unwrap();
        assert_eq!(value["custom"], "payload");
        // Not wrapped in {"fields": ...}
        assert!(value.get("fields").is_none());
    }

    #[test]
    fn test_body_and_body_file_conflict() {
        let result = Cli::try_parse_from([
            "shrug",
            "jira",
            "issue",
            "create",
            "-s",
            "Test",
            "--project",
            "TEAM",
            "--type",
            "Bug",
            "--body",
            "some text",
            "--body-file",
            "file.md",
        ]);
        assert!(result.is_err(), "--body and --body-file should conflict");
    }
}
