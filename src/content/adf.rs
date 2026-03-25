//! Atlassian Document Format (ADF) terminal renderer.
//!
//! Converts ADF JSON content to readable terminal text. Supports common
//! node types: paragraph, heading, bulletList, orderedList, codeBlock,
//! blockquote, rule, text with marks. Unknown types are silently skipped.

/// Render ADF JSON to terminal text.
///
/// If the value is not a valid ADF document (object with "type": "doc"),
/// returns the compact JSON representation instead.
pub fn render_adf(adf: &serde_json::Value, color_enabled: bool) -> String {
    // Must be an object with type: "doc"
    let obj = match adf.as_object() {
        Some(o) => o,
        None => return adf.to_string(),
    };

    if obj.get("type").and_then(|v| v.as_str()) != Some("doc") {
        return adf.to_string();
    }

    let content = match obj.get("content").and_then(|v| v.as_array()) {
        Some(c) => c,
        None => return String::new(),
    };

    let mut output = String::new();
    render_nodes(content, &mut output, color_enabled, 0);
    output.trim_end().to_string()
}

/// Check if a JSON value is ADF content.
pub fn is_adf(value: &serde_json::Value) -> bool {
    value
        .as_object()
        .and_then(|o| o.get("type"))
        .and_then(|v| v.as_str())
        == Some("doc")
        && value.get("content").and_then(|v| v.as_array()).is_some()
}

fn render_nodes(
    nodes: &[serde_json::Value],
    output: &mut String,
    color_enabled: bool,
    indent: usize,
) {
    for node in nodes {
        let node_type = match node.get("type").and_then(|v| v.as_str()) {
            Some(t) => t,
            None => continue,
        };

        match node_type {
            "paragraph" => render_paragraph(node, output, color_enabled, indent),
            "heading" => render_heading(node, output, color_enabled),
            "bulletList" => render_bullet_list(node, output, color_enabled, indent),
            "orderedList" => render_ordered_list(node, output, color_enabled, indent),
            "codeBlock" => render_code_block(node, output),
            "blockquote" => render_blockquote(node, output, color_enabled),
            "rule" => output.push_str("---\n\n"),
            "text" => render_text(node, output, color_enabled),
            "hardBreak" => output.push('\n'),
            "mention" => render_mention(node, output),
            "emoji" => render_emoji(node, output),
            _ => {} // Unknown types silently skipped
        }
    }
}

fn render_paragraph(
    node: &serde_json::Value,
    output: &mut String,
    color_enabled: bool,
    indent: usize,
) {
    let prefix = " ".repeat(indent);
    if indent > 0 {
        output.push_str(&prefix);
    }
    if let Some(content) = node.get("content").and_then(|v| v.as_array()) {
        render_inline(content, output, color_enabled);
    }
    output.push_str("\n\n");
}

fn render_heading(node: &serde_json::Value, output: &mut String, color_enabled: bool) {
    let level = node
        .get("attrs")
        .and_then(|a| a.get("level"))
        .and_then(|l| l.as_u64())
        .unwrap_or(1) as usize;

    let prefix = "#".repeat(level.min(6));
    output.push_str(&prefix);
    output.push(' ');

    if let Some(content) = node.get("content").and_then(|v| v.as_array()) {
        render_inline(content, output, color_enabled);
    }
    output.push_str("\n\n");
}

fn render_bullet_list(
    node: &serde_json::Value,
    output: &mut String,
    color_enabled: bool,
    indent: usize,
) {
    if let Some(content) = node.get("content").and_then(|v| v.as_array()) {
        for item in content {
            if item.get("type").and_then(|v| v.as_str()) == Some("listItem") {
                let prefix = " ".repeat(indent);
                output.push_str(&prefix);
                output.push_str("- ");
                if let Some(item_content) = item.get("content").and_then(|v| v.as_array()) {
                    render_list_item_content(item_content, output, color_enabled, indent + 2);
                }
            }
        }
    }
}

fn render_ordered_list(
    node: &serde_json::Value,
    output: &mut String,
    color_enabled: bool,
    indent: usize,
) {
    if let Some(content) = node.get("content").and_then(|v| v.as_array()) {
        for (i, item) in content.iter().enumerate() {
            if item.get("type").and_then(|v| v.as_str()) == Some("listItem") {
                let prefix = " ".repeat(indent);
                output.push_str(&format!("{}{}. ", prefix, i + 1));
                if let Some(item_content) = item.get("content").and_then(|v| v.as_array()) {
                    render_list_item_content(item_content, output, color_enabled, indent + 3);
                }
            }
        }
    }
}

fn render_list_item_content(
    content: &[serde_json::Value],
    output: &mut String,
    color_enabled: bool,
    indent: usize,
) {
    for node in content {
        let node_type = match node.get("type").and_then(|v| v.as_str()) {
            Some(t) => t,
            None => continue,
        };
        match node_type {
            "paragraph" => {
                if let Some(inline) = node.get("content").and_then(|v| v.as_array()) {
                    render_inline(inline, output, color_enabled);
                }
                output.push('\n');
            }
            "bulletList" => {
                output.push('\n');
                render_bullet_list(node, output, color_enabled, indent);
            }
            "orderedList" => {
                output.push('\n');
                render_ordered_list(node, output, color_enabled, indent);
            }
            _ => {}
        }
    }
}

fn render_code_block(node: &serde_json::Value, output: &mut String) {
    if let Some(content) = node.get("content").and_then(|v| v.as_array()) {
        for child in content {
            if let Some(text) = child.get("text").and_then(|v| v.as_str()) {
                for line in text.lines() {
                    output.push_str("    ");
                    output.push_str(line);
                    output.push('\n');
                }
            }
        }
    }
    output.push('\n');
}

fn render_blockquote(node: &serde_json::Value, output: &mut String, color_enabled: bool) {
    if let Some(content) = node.get("content").and_then(|v| v.as_array()) {
        let mut inner = String::new();
        render_nodes(content, &mut inner, color_enabled, 0);
        for line in inner.trim_end().lines() {
            output.push_str("> ");
            output.push_str(line);
            output.push('\n');
        }
        output.push('\n');
    }
}

fn render_inline(nodes: &[serde_json::Value], output: &mut String, color_enabled: bool) {
    for node in nodes {
        let node_type = match node.get("type").and_then(|v| v.as_str()) {
            Some(t) => t,
            None => continue,
        };
        match node_type {
            "text" => render_text(node, output, color_enabled),
            "hardBreak" => output.push('\n'),
            "mention" => render_mention(node, output),
            "emoji" => render_emoji(node, output),
            _ => {}
        }
    }
}

fn render_text(node: &serde_json::Value, output: &mut String, color_enabled: bool) {
    let text = match node.get("text").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return,
    };

    let marks = node.get("marks").and_then(|v| v.as_array());

    if !color_enabled || marks.is_none() {
        // Check for link mark even without colour
        if let Some(marks_arr) = marks {
            for mark in marks_arr {
                if mark.get("type").and_then(|v| v.as_str()) == Some("link") {
                    if let Some(href) = mark
                        .get("attrs")
                        .and_then(|a| a.get("href"))
                        .and_then(|v| v.as_str())
                    {
                        output.push_str(text);
                        output.push_str(" (");
                        output.push_str(href);
                        output.push(')');
                        return;
                    }
                }
            }
        }
        output.push_str(text);
        return;
    }

    let marks_arr = marks.unwrap();
    let mut has_link = false;
    let mut link_href = String::new();

    // Collect marks
    let mut prefix = String::new();
    let mut suffix = String::new();

    for mark in marks_arr {
        let mark_type = match mark.get("type").and_then(|v| v.as_str()) {
            Some(t) => t,
            None => continue,
        };
        match mark_type {
            "strong" => {
                prefix.push_str("\x1b[1m");
                suffix = format!("\x1b[0m{}", suffix);
            }
            "em" => {
                prefix.push_str("\x1b[3m");
                suffix = format!("\x1b[0m{}", suffix);
            }
            "code" => {
                prefix.push_str("\x1b[2m");
                suffix = format!("\x1b[0m{}", suffix);
            }
            "link" => {
                has_link = true;
                if let Some(href) = mark
                    .get("attrs")
                    .and_then(|a| a.get("href"))
                    .and_then(|v| v.as_str())
                {
                    link_href = href.to_string();
                }
            }
            _ => {}
        }
    }

    output.push_str(&prefix);
    output.push_str(text);
    output.push_str(&suffix);

    if has_link && !link_href.is_empty() {
        output.push_str(" (");
        output.push_str(&link_href);
        output.push(')');
    }
}

fn render_mention(node: &serde_json::Value, output: &mut String) {
    let attrs = match node.get("attrs") {
        Some(a) => a,
        None => return,
    };

    if let Some(text) = attrs.get("text").and_then(|v| v.as_str()) {
        output.push_str(text);
    } else if let Some(id) = attrs.get("id").and_then(|v| v.as_str()) {
        output.push('@');
        output.push_str(id);
    }
}

fn render_emoji(node: &serde_json::Value, output: &mut String) {
    let attrs = match node.get("attrs") {
        Some(a) => a,
        None => return,
    };

    if let Some(text) = attrs.get("text").and_then(|v| v.as_str()) {
        output.push_str(text);
    } else if let Some(short_name) = attrs.get("shortName").and_then(|v| v.as_str()) {
        output.push_str(short_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_doc(content: Vec<serde_json::Value>) -> serde_json::Value {
        serde_json::json!({
            "type": "doc",
            "version": 1,
            "content": content
        })
    }

    #[test]
    fn render_paragraph_produces_plain_text() {
        let doc = make_doc(vec![serde_json::json!({
            "type": "paragraph",
            "content": [{"type": "text", "text": "Hello world"}]
        })]);
        let output = render_adf(&doc, false);
        assert_eq!(output, "Hello world");
    }

    #[test]
    fn render_heading_produces_hash_prefix() {
        let doc = make_doc(vec![serde_json::json!({
            "type": "heading",
            "attrs": {"level": 2},
            "content": [{"type": "text", "text": "Title"}]
        })]);
        let output = render_adf(&doc, false);
        assert_eq!(output, "## Title");
    }

    #[test]
    fn render_bullet_list_produces_dash_markers() {
        let doc = make_doc(vec![serde_json::json!({
            "type": "bulletList",
            "content": [
                {"type": "listItem", "content": [
                    {"type": "paragraph", "content": [{"type": "text", "text": "First"}]}
                ]},
                {"type": "listItem", "content": [
                    {"type": "paragraph", "content": [{"type": "text", "text": "Second"}]}
                ]}
            ]
        })]);
        let output = render_adf(&doc, false);
        assert!(output.contains("- First"));
        assert!(output.contains("- Second"));
    }

    #[test]
    fn render_ordered_list_produces_numbered_markers() {
        let doc = make_doc(vec![serde_json::json!({
            "type": "orderedList",
            "content": [
                {"type": "listItem", "content": [
                    {"type": "paragraph", "content": [{"type": "text", "text": "Alpha"}]}
                ]},
                {"type": "listItem", "content": [
                    {"type": "paragraph", "content": [{"type": "text", "text": "Beta"}]}
                ]}
            ]
        })]);
        let output = render_adf(&doc, false);
        assert!(output.contains("1. Alpha"));
        assert!(output.contains("2. Beta"));
    }

    #[test]
    fn render_code_block_produces_indented_output() {
        let doc = make_doc(vec![serde_json::json!({
            "type": "codeBlock",
            "content": [{"type": "text", "text": "fn main() {\n    println!(\"hello\");\n}"}]
        })]);
        let output = render_adf(&doc, false);
        assert!(output.contains("    fn main() {"));
        assert!(output.contains("        println!(\"hello\");"));
    }

    #[test]
    fn render_text_marks_with_color_enabled() {
        let doc = make_doc(vec![serde_json::json!({
            "type": "paragraph",
            "content": [{
                "type": "text",
                "text": "bold text",
                "marks": [{"type": "strong"}]
            }]
        })]);
        let output = render_adf(&doc, true);
        assert!(output.contains("\x1b[1m"), "Should contain ANSI bold");
        assert!(output.contains("bold text"));
    }

    #[test]
    fn render_text_marks_without_color() {
        let doc = make_doc(vec![serde_json::json!({
            "type": "paragraph",
            "content": [{
                "type": "text",
                "text": "bold text",
                "marks": [{"type": "strong"}]
            }]
        })]);
        let output = render_adf(&doc, false);
        assert!(!output.contains("\x1b["), "Should not contain ANSI codes");
        assert!(output.contains("bold text"));
    }

    #[test]
    fn render_link_appends_url() {
        let doc = make_doc(vec![serde_json::json!({
            "type": "paragraph",
            "content": [{
                "type": "text",
                "text": "click here",
                "marks": [{"type": "link", "attrs": {"href": "https://example.com"}}]
            }]
        })]);
        let output = render_adf(&doc, false);
        assert!(output.contains("click here (https://example.com)"));
    }

    #[test]
    fn render_non_adf_returns_json_string() {
        let value = serde_json::json!({"key": "value"});
        let output = render_adf(&value, false);
        assert!(output.contains("key"));
        assert!(output.contains("value"));
    }

    #[test]
    fn render_unknown_node_types_skipped_gracefully() {
        let doc = make_doc(vec![
            serde_json::json!({"type": "unknownNode", "content": []}),
            serde_json::json!({
                "type": "paragraph",
                "content": [{"type": "text", "text": "visible"}]
            }),
        ]);
        let output = render_adf(&doc, false);
        assert!(output.contains("visible"));
        assert!(!output.contains("unknownNode"));
    }

    #[test]
    fn is_adf_detects_doc_type() {
        let adf = serde_json::json!({"type": "doc", "version": 1, "content": []});
        assert!(is_adf(&adf));

        let not_adf = serde_json::json!({"type": "paragraph"});
        assert!(!is_adf(&not_adf));

        let plain = serde_json::json!("hello");
        assert!(!is_adf(&plain));
    }

    #[test]
    fn render_rule_produces_horizontal_line() {
        let doc = make_doc(vec![serde_json::json!({"type": "rule"})]);
        let output = render_adf(&doc, false);
        assert_eq!(output, "---");
    }

    #[test]
    fn render_blockquote_prefixes_with_gt() {
        let doc = make_doc(vec![serde_json::json!({
            "type": "blockquote",
            "content": [{
                "type": "paragraph",
                "content": [{"type": "text", "text": "quoted text"}]
            }]
        })]);
        let output = render_adf(&doc, false);
        assert!(output.contains("> quoted text"));
    }
}
