use std::io::Write;

use crate::adf;
use crate::cli::{ColorChoice, OutputFormat};

/// Format an API response body according to the chosen output format.
///
/// If the body is not valid JSON, returns it unchanged (Atlassian APIs
/// may return HTML error pages or plain text).
///
/// When `fields` is provided and format is Table or Csv, only those
/// fields are shown in the output.
pub fn format_response(
    body: &str,
    format: &OutputFormat,
    _is_tty: bool,
    color_enabled: bool,
    fields: Option<&[String]>,
) -> String {
    let json: serde_json::Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(_) => return body.to_string(),
    };

    // Apply field filtering for table/CSV only
    let json = match (fields, format) {
        (Some(f), OutputFormat::Table | OutputFormat::Csv) => filter_fields(&json, f),
        _ => json,
    };

    match format {
        OutputFormat::Json => format_json(&json),
        OutputFormat::Table => format_table(&json, color_enabled),
        OutputFormat::Yaml => format_yaml(&json),
        OutputFormat::Csv => format_csv_with_fields(&json, fields),
        OutputFormat::Plain => format_plain(&json),
    }
}

/// Print output, optionally through a pager.
///
/// When `use_pager` is true and stdout is a TTY, pipes output through
/// $PAGER (defaulting to "less -R -F -X"). Falls back to direct print
/// on pager spawn failure.
pub fn print_with_pager(output: &str, use_pager: bool, is_tty: bool) {
    if !use_pager || !is_tty {
        println!("{}", output);
        return;
    }

    let pager_cmd = std::env::var("PAGER").unwrap_or_else(|_| "less -R -F -X".to_string());
    let parts: Vec<&str> = pager_cmd.split_whitespace().collect();
    if parts.is_empty() {
        println!("{}", output);
        return;
    }

    let mut cmd = std::process::Command::new(parts[0]);
    for arg in &parts[1..] {
        cmd.arg(arg);
    }
    cmd.stdin(std::process::Stdio::piped());

    match cmd.spawn() {
        Ok(mut child) => {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(output.as_bytes());
                let _ = stdin.write_all(b"\n");
            }
            let _ = child.wait();
        }
        Err(_) => {
            println!("{}", output);
        }
    }
}

/// Filter a JSON value to retain only the specified fields.
///
/// For wrapper objects (containing issues/values/results arrays),
/// filters the array elements. Preserves the field order from the
/// `fields` parameter.
pub fn filter_fields(json: &serde_json::Value, fields: &[String]) -> serde_json::Value {
    // Check for wrapper objects first
    for key in &["issues", "values", "results"] {
        if let Some(arr) = json.get(key).and_then(|v| v.as_array()) {
            let filtered: Vec<serde_json::Value> =
                arr.iter().map(|item| filter_object(item, fields)).collect();
            return serde_json::Value::Array(filtered);
        }
    }

    // Top-level array
    if let Some(arr) = json.as_array() {
        let filtered: Vec<serde_json::Value> =
            arr.iter().map(|item| filter_object(item, fields)).collect();
        return serde_json::Value::Array(filtered);
    }

    // Single object
    filter_object(json, fields)
}

fn filter_object(value: &serde_json::Value, fields: &[String]) -> serde_json::Value {
    if let Some(obj) = value.as_object() {
        let mut filtered = serde_json::Map::new();
        for field in fields {
            if let Some(v) = obj.get(field) {
                filtered.insert(field.clone(), v.clone());
            }
        }
        serde_json::Value::Object(filtered)
    } else {
        value.clone()
    }
}

/// Resolve the effective output format based on TTY detection.
///
/// When the default (Table) is selected and stdout is not a TTY,
/// automatically switch to JSON for machine consumption.
pub fn resolve_format(explicit: &OutputFormat, is_tty: bool) -> OutputFormat {
    match explicit {
        OutputFormat::Table if !is_tty => OutputFormat::Json,
        other => other.clone(),
    }
}

/// Determine whether colour output should be enabled.
pub fn should_use_color(choice: &ColorChoice, is_tty: bool) -> bool {
    match choice {
        ColorChoice::Always => true,
        ColorChoice::Never => false,
        ColorChoice::Auto => is_tty && std::env::var("NO_COLOR").is_err(),
    }
}

// --- Format implementations ---

fn format_json(json: &serde_json::Value) -> String {
    serde_json::to_string_pretty(json).unwrap_or_else(|_| json.to_string())
}

fn format_table(json: &serde_json::Value, color_enabled: bool) -> String {
    use comfy_table::{presets, Table};

    // Try to find a results array inside a wrapper object
    if let Some(arr) = extract_results_array(json) {
        return format_array_table(arr, color_enabled);
    }

    // Top-level array
    if let Some(arr) = json.as_array() {
        if !arr.is_empty() {
            return format_array_table(arr, color_enabled);
        }
    }

    // Single object: key/value table
    if let Some(obj) = json.as_object() {
        let mut table = Table::new();
        table.load_preset(presets::UTF8_FULL_CONDENSED);
        if !color_enabled {
            table.force_no_tty();
        }
        table.set_header(vec!["Field", "Value"]);
        for (key, value) in obj {
            table.add_row(vec![key.clone(), truncate_value(value)]);
        }
        return table.to_string();
    }

    // Fallback for scalars
    json.to_string()
}

fn format_array_table(arr: &[serde_json::Value], color_enabled: bool) -> String {
    use comfy_table::{presets, Table};

    if arr.is_empty() {
        return String::from("(empty)");
    }

    // Collect all keys from all objects for consistent columns
    let mut all_keys: Vec<String> = Vec::new();
    for item in arr {
        if let Some(obj) = item.as_object() {
            for key in obj.keys() {
                if !all_keys.contains(key) {
                    all_keys.push(key.clone());
                }
            }
        }
    }

    if all_keys.is_empty() {
        // Array of non-objects
        let mut table = Table::new();
        table.load_preset(presets::UTF8_FULL_CONDENSED);
        if !color_enabled {
            table.force_no_tty();
        }
        table.set_header(vec!["Value"]);
        for item in arr {
            table.add_row(vec![truncate_value(item)]);
        }
        return table.to_string();
    }

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL_CONDENSED);
    if !color_enabled {
        table.force_no_tty();
    }
    table.set_header(all_keys.iter().map(|k| k.as_str()));

    for item in arr {
        let row: Vec<String> = all_keys
            .iter()
            .map(|key| item.get(key).map(truncate_value).unwrap_or_default())
            .collect();
        table.add_row(row);
    }

    table.to_string()
}

fn format_yaml(json: &serde_json::Value) -> String {
    serde_yaml_ng::to_string(json).unwrap_or_else(|_| format_json(json))
}

/// Format CSV with optional field ordering from --fields.
fn format_csv_with_fields(json: &serde_json::Value, fields: Option<&[String]>) -> String {
    if let Some(field_list) = fields {
        // When --fields is specified, use that order (data already filtered)
        return format_csv_ordered(json, field_list);
    }
    format_csv(json)
}

/// Format CSV using a specific column order.
fn format_csv_ordered(json: &serde_json::Value, columns: &[String]) -> String {
    let arr = if let Some(arr) = json.as_array() {
        arr.clone()
    } else {
        return format_json(json);
    };

    if arr.is_empty() || columns.is_empty() {
        return String::new();
    }

    let mut writer = csv::Writer::from_writer(Vec::new());
    writer.write_record(columns).ok();

    for item in &arr {
        let row: Vec<String> = columns
            .iter()
            .map(|key| item.get(key).map(value_to_csv_cell).unwrap_or_default())
            .collect();
        writer.write_record(&row).ok();
    }

    writer.flush().ok();
    String::from_utf8(writer.into_inner().unwrap_or_default())
        .unwrap_or_default()
        .trim_end()
        .to_string()
}

fn format_csv(json: &serde_json::Value) -> String {
    // Find the array to render
    let arr = if let Some(results) = extract_results_array(json) {
        results.to_vec()
    } else if let Some(arr) = json.as_array() {
        arr.clone()
    } else {
        // Not an array — fall back to JSON
        return format_json(json);
    };

    if arr.is_empty() {
        return String::new();
    }

    // Collect all keys, sorted alphabetically for deterministic output
    let mut all_keys: Vec<String> = Vec::new();
    for item in &arr {
        if let Some(obj) = item.as_object() {
            for key in obj.keys() {
                if !all_keys.contains(key) {
                    all_keys.push(key.clone());
                }
            }
        }
    }
    all_keys.sort();

    if all_keys.is_empty() {
        return format_json(json);
    }

    let mut writer = csv::Writer::from_writer(Vec::new());

    // Header row
    writer.write_record(&all_keys).ok();

    // Data rows
    for item in &arr {
        let row: Vec<String> = all_keys
            .iter()
            .map(|key| item.get(key).map(value_to_csv_cell).unwrap_or_default())
            .collect();
        writer.write_record(&row).ok();
    }

    writer.flush().ok();
    String::from_utf8(writer.into_inner().unwrap_or_default())
        .unwrap_or_default()
        .trim_end()
        .to_string()
}

fn format_plain(json: &serde_json::Value) -> String {
    match json {
        serde_json::Value::Object(obj) => {
            let mut lines = Vec::new();
            for (key, value) in obj {
                let display = if adf::is_adf(value) {
                    adf::render_adf(value, false)
                } else {
                    match value {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Null => String::from("null"),
                        serde_json::Value::Bool(b) => b.to_string(),
                        serde_json::Value::Number(n) => n.to_string(),
                        other => other.to_string(),
                    }
                };
                lines.push(format!("{}: {}", key, display));
            }
            lines.join("\n")
        }
        serde_json::Value::Array(arr) => arr
            .iter()
            .map(|item| match item {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            })
            .collect::<Vec<_>>()
            .join("\n"),
        other => other.to_string(),
    }
}

// --- Helpers ---

/// Extract a known results array from a wrapper object.
fn extract_results_array(json: &serde_json::Value) -> Option<&[serde_json::Value]> {
    for key in &["issues", "values", "results"] {
        if let Some(arr) = json.get(key).and_then(|v| v.as_array()) {
            return Some(arr.as_slice());
        }
    }
    None
}

/// Truncate a JSON value to a displayable string for table cells.
fn truncate_value(value: &serde_json::Value) -> String {
    // Render ADF content as text instead of raw JSON
    if adf::is_adf(value) {
        let rendered = adf::render_adf(value, false);
        let oneline = rendered.replace('\n', " ").trim().to_string();
        if oneline.len() > 60 {
            return format!("{}...", &oneline[..57]);
        }
        return oneline;
    }

    let s = match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::from("null"),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        other => other.to_string(),
    };
    if s.len() > 60 {
        format!("{}...", &s[..57])
    } else {
        s
    }
}

/// Convert a JSON value to a CSV cell string.
fn value_to_csv_cell(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_object() -> serde_json::Value {
        serde_json::json!({
            "id": 123,
            "key": "TEST-1",
            "summary": "Fix the login bug",
            "status": "Open"
        })
    }

    fn sample_array() -> serde_json::Value {
        serde_json::json!([
            {"id": 1, "name": "Alice", "role": "admin"},
            {"id": 2, "name": "Bob", "role": "user"}
        ])
    }

    fn sample_wrapper() -> serde_json::Value {
        serde_json::json!({
            "total": 2,
            "startAt": 0,
            "maxResults": 50,
            "issues": [
                {"id": "10001", "key": "PROJ-1", "summary": "First issue"},
                {"id": "10002", "key": "PROJ-2", "summary": "Second issue"}
            ]
        })
    }

    #[test]
    fn format_json_pretty_prints() {
        let json = sample_object();
        let output = format_json(&json);
        assert!(output.contains('\n'), "Should be multi-line");
        assert!(output.contains("  "), "Should have indentation");
        assert!(output.contains("\"key\": \"TEST-1\""));
    }

    #[test]
    fn format_table_renders_object_as_key_value() {
        let json = sample_object();
        let output = format_table(&json, false);
        assert!(output.contains("Field"), "Should have Field header");
        assert!(output.contains("Value"), "Should have Value header");
        assert!(output.contains("TEST-1"));
        assert!(output.contains("summary"));
    }

    #[test]
    fn format_table_renders_array_as_columnar() {
        let json = sample_array();
        let output = format_table(&json, false);
        assert!(output.contains("name"), "Should have name column");
        assert!(output.contains("Alice"));
        assert!(output.contains("Bob"));
    }

    #[test]
    fn format_table_extracts_issues_array() {
        let json = sample_wrapper();
        let output = format_table(&json, false);
        // Should render the issues array, not the wrapper object
        assert!(output.contains("PROJ-1"));
        assert!(output.contains("PROJ-2"));
        assert!(output.contains("key"), "Should have key column");
    }

    #[test]
    fn format_yaml_produces_valid_yaml() {
        let json = sample_object();
        let output = format_yaml(&json);
        assert!(output.contains("key: TEST-1"));
        assert!(output.contains("summary: Fix the login bug"));
    }

    #[test]
    fn format_csv_produces_header_and_data_rows() {
        let json = sample_array();
        let output = format_csv(&json);
        let lines: Vec<&str> = output.lines().collect();
        assert!(lines.len() >= 3, "Should have header + 2 data rows");
        // Headers should be sorted alphabetically
        assert!(
            lines[0].starts_with("id,"),
            "First column should be 'id' (sorted): got '{}'",
            lines[0]
        );
        assert!(output.contains("Alice"));
        assert!(output.contains("Bob"));
    }

    #[test]
    fn format_csv_sorted_columns() {
        let json = serde_json::json!([
            {"zebra": 1, "alpha": 2, "middle": 3}
        ]);
        let output = format_csv(&json);
        let header = output.lines().next().unwrap();
        assert_eq!(
            header, "alpha,middle,zebra",
            "Columns should be sorted alphabetically"
        );
    }

    #[test]
    fn format_csv_falls_back_to_json_for_non_array() {
        let json = sample_object();
        let output = format_csv(&json);
        // Should fall back to pretty JSON
        assert!(output.contains("\"key\": \"TEST-1\""));
    }

    #[test]
    fn format_plain_renders_key_value_pairs() {
        let json = sample_object();
        let output = format_plain(&json);
        assert!(output.contains("key: TEST-1"));
        assert!(output.contains("summary: Fix the login bug"));
        assert!(output.contains("id: 123"));
    }

    #[test]
    fn format_plain_renders_array_items() {
        let json = serde_json::json!(["one", "two", "three"]);
        let output = format_plain(&json);
        assert!(output.contains("one"));
        assert!(output.contains("two"));
        assert!(output.contains("three"));
    }

    #[test]
    fn resolve_format_returns_json_when_table_not_tty() {
        let format = resolve_format(&OutputFormat::Table, false);
        assert_eq!(format, OutputFormat::Json);
    }

    #[test]
    fn resolve_format_returns_table_when_table_and_tty() {
        let format = resolve_format(&OutputFormat::Table, true);
        assert_eq!(format, OutputFormat::Table);
    }

    #[test]
    fn resolve_format_preserves_explicit_json() {
        let format = resolve_format(&OutputFormat::Json, true);
        assert_eq!(format, OutputFormat::Json);
    }

    #[test]
    fn resolve_format_preserves_explicit_csv() {
        let format = resolve_format(&OutputFormat::Csv, false);
        assert_eq!(format, OutputFormat::Csv);
    }

    #[test]
    fn should_use_color_never_returns_false() {
        assert!(!should_use_color(&ColorChoice::Never, true));
        assert!(!should_use_color(&ColorChoice::Never, false));
    }

    #[test]
    fn should_use_color_always_returns_true() {
        assert!(should_use_color(&ColorChoice::Always, true));
        assert!(should_use_color(&ColorChoice::Always, false));
    }

    #[test]
    fn format_response_returns_raw_body_for_invalid_json() {
        let body = "<html><body>Error</body></html>";
        let output = format_response(body, &OutputFormat::Json, true, false, None);
        assert_eq!(
            output, body,
            "Should return raw body when JSON parsing fails"
        );
    }

    #[test]
    fn format_response_returns_raw_body_for_plain_text() {
        let body = "Not Found";
        let output = format_response(body, &OutputFormat::Table, true, false, None);
        assert_eq!(output, body);
    }

    #[test]
    fn truncate_value_truncates_long_strings() {
        let long_str = "a".repeat(100);
        let json_val = serde_json::Value::String(long_str);
        let truncated = truncate_value(&json_val);
        assert_eq!(truncated.len(), 60);
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn truncate_value_preserves_short_strings() {
        let json_val = serde_json::Value::String("short".to_string());
        assert_eq!(truncate_value(&json_val), "short");
    }

    #[test]
    fn format_csv_extracts_issues_from_wrapper() {
        let json = sample_wrapper();
        let output = format_csv(&json);
        assert!(output.contains("PROJ-1"));
        assert!(output.contains("PROJ-2"));
        // Should have header row with issue fields, not wrapper fields
        let header = output.lines().next().unwrap();
        assert!(
            header.contains("key"),
            "Should have 'key' column from issues"
        );
    }

    // --- filter_fields tests ---

    #[test]
    fn filter_fields_retains_specified_keys_in_order() {
        let json = sample_object();
        let fields = vec!["summary".to_string(), "key".to_string()];
        let filtered = filter_fields(&json, &fields);
        let obj = filtered.as_object().unwrap();
        let keys: Vec<&String> = obj.keys().collect();
        assert_eq!(keys, vec!["summary", "key"]);
    }

    #[test]
    fn filter_fields_works_on_wrapper_objects() {
        let json = sample_wrapper();
        let fields = vec!["key".to_string(), "summary".to_string()];
        let filtered = filter_fields(&json, &fields);
        let arr = filtered.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        let first = arr[0].as_object().unwrap();
        assert!(first.contains_key("key"));
        assert!(first.contains_key("summary"));
        assert!(
            !first.contains_key("id"),
            "Should not contain unselected fields"
        );
    }

    #[test]
    fn filter_fields_returns_unchanged_for_non_object() {
        let json = serde_json::json!(42);
        let fields = vec!["key".to_string()];
        let filtered = filter_fields(&json, &fields);
        assert_eq!(filtered, json);
    }

    #[test]
    fn format_response_with_fields_filters_table() {
        let body =
            r#"[{"id":1,"name":"Alice","role":"admin"},{"id":2,"name":"Bob","role":"user"}]"#;
        let fields = vec!["name".to_string()];
        let output = format_response(body, &OutputFormat::Table, true, false, Some(&fields));
        assert!(output.contains("Alice"));
        assert!(output.contains("Bob"));
        // Should not contain "admin" or "user" since "role" is not in fields
        assert!(
            !output.contains("admin"),
            "role column should be filtered out"
        );
    }

    #[test]
    fn format_response_with_fields_does_not_filter_json() {
        let body = r#"{"id":1,"name":"Alice","role":"admin"}"#;
        let fields = vec!["name".to_string()];
        let output = format_response(body, &OutputFormat::Json, true, false, Some(&fields));
        // JSON output should contain all fields
        assert!(output.contains("admin"), "JSON should not be filtered");
        assert!(output.contains("role"));
    }

    #[test]
    fn print_with_pager_prints_directly_when_disabled() {
        // This just verifies it doesn't panic when pager is disabled
        print_with_pager("test output", false, false);
    }

    #[test]
    fn adf_in_table_cell_rendered_as_text() {
        let json = serde_json::json!({
            "key": "TEST-1",
            "description": {
                "type": "doc",
                "version": 1,
                "content": [{"type": "paragraph", "content": [{"type": "text", "text": "A bug report"}]}]
            }
        });
        let output = format_table(&json, false);
        assert!(
            output.contains("A bug report"),
            "ADF should be rendered as text in table"
        );
        assert!(
            !output.contains("\"type\""),
            "ADF JSON structure should not appear"
        );
    }

    #[test]
    fn adf_in_plain_output_rendered_as_text() {
        let json = serde_json::json!({
            "key": "TEST-1",
            "description": {
                "type": "doc",
                "version": 1,
                "content": [{"type": "paragraph", "content": [{"type": "text", "text": "Plain ADF"}]}]
            }
        });
        let output = format_plain(&json);
        assert!(output.contains("description: Plain ADF"));
    }

    #[test]
    fn format_csv_with_fields_respects_field_order() {
        let json = serde_json::json!([
            {"c": 3, "a": 1, "b": 2}
        ]);
        let fields = vec!["b".to_string(), "a".to_string()];
        let filtered = filter_fields(&json, &fields);
        let output = format_csv_with_fields(&filtered, Some(&fields));
        let header = output.lines().next().unwrap();
        assert_eq!(
            header, "b,a",
            "CSV should respect --fields order, not alphabetical"
        );
    }
}
