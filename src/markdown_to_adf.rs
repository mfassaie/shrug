//! Markdown to Atlassian Document Format (ADF) converter.
//!
//! Converts Markdown input into ADF JSON suitable for Atlassian Cloud APIs.
//! Uses pulldown-cmark to parse Markdown into events, then transforms those
//! events into an ADF node tree.

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

use crate::error::ShrugError;

/// Convert a Markdown string to an ADF JSON document.
///
/// Returns a `serde_json::Value` with `{"type": "doc", "version": 1, "content": [...]}`.
/// Handles paragraphs, headings, bold, italic, code spans, bullet lists,
/// ordered lists, code blocks, blockquotes, links, horizontal rules, and hard breaks.
pub fn markdown_to_adf(markdown: &str) -> serde_json::Value {
    let options = Options::ENABLE_STRIKETHROUGH;
    let parser = Parser::new_ext(markdown, options);

    let mut builder = AdfBuilder::new();
    for event in parser {
        builder.process_event(event);
    }
    builder.finish()
}

/// Convert Markdown fields in a JSON body to ADF.
///
/// Parses the body as JSON, recursively walks all objects looking for known
/// ADF field keys ("description", "body", "comment") at any nesting depth.
/// Only converts values that are plain strings (skips objects/arrays).
/// Returns the modified JSON string.
pub fn convert_body_markdown(body: &str) -> Result<String, ShrugError> {
    let mut json: serde_json::Value = serde_json::from_str(body)
        .map_err(|e| ShrugError::UsageError(format!("Invalid JSON body: {}", e)))?;

    convert_value_recursive(&mut json);

    serde_json::to_string(&json)
        .map_err(|e| ShrugError::UsageError(format!("Failed to serialise converted body: {}", e)))
}

/// Known field names whose string values should be converted from Markdown to ADF.
const ADF_FIELD_NAMES: &[&str] = &["description", "body", "comment"];

fn convert_value_recursive(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map.iter_mut() {
                if ADF_FIELD_NAMES.contains(&key.as_str()) {
                    if let serde_json::Value::String(s) = val {
                        *val = markdown_to_adf(s);
                    }
                }
                // Recurse into all values regardless of key name
                convert_value_recursive(val);
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr.iter_mut() {
                convert_value_recursive(item);
            }
        }
        _ => {}
    }
}

// --- ADF Builder ---

/// Internal builder that accumulates ADF nodes from pulldown-cmark events.
struct AdfBuilder {
    /// Stack of container nodes. The bottom is always the doc content array.
    stack: Vec<ContainerNode>,
    /// Active inline marks (bold, italic, etc.)
    marks: Vec<Mark>,
    /// Current link href, if inside a link
    link_href: Option<String>,
}

#[derive(Debug)]
enum ContainerNode {
    /// Top-level doc content
    Doc(Vec<serde_json::Value>),
    /// Paragraph collecting inline content
    Paragraph(Vec<serde_json::Value>),
    /// Heading collecting inline content
    Heading(Vec<serde_json::Value>),
    /// Bullet list collecting list items
    BulletList(Vec<serde_json::Value>),
    /// Ordered list collecting list items
    OrderedList(Vec<serde_json::Value>),
    /// Single list item containing block content
    ListItem(Vec<serde_json::Value>),
    /// Code block with optional language
    CodeBlock(Option<String>, String),
    /// Blockquote containing block content
    Blockquote(Vec<serde_json::Value>),
}

#[derive(Debug, Clone)]
enum Mark {
    Strong,
    Em,
    Link(String),
}

impl AdfBuilder {
    fn new() -> Self {
        Self {
            stack: vec![ContainerNode::Doc(Vec::new())],
            marks: Vec::new(),
            link_href: None,
        }
    }

    fn process_event(&mut self, event: Event) {
        match event {
            Event::Start(tag) => self.start_tag(tag),
            Event::End(tag) => self.end_tag(tag),
            Event::Text(text) => self.add_text(&text),
            Event::Code(code) => self.add_inline_code(&code),
            Event::SoftBreak => self.add_text(" "),
            Event::HardBreak => self.add_node(serde_json::json!({"type": "hardBreak"})),
            Event::Rule => self.add_block_node(serde_json::json!({"type": "rule"})),
            _ => {} // TaskListMarker, FootnoteReference, etc. — skip
        }
    }

    fn start_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Paragraph => {
                self.stack.push(ContainerNode::Paragraph(Vec::new()));
            }
            Tag::Heading { .. } => {
                self.stack.push(ContainerNode::Heading(Vec::new()));
            }
            Tag::List(None) => {
                self.stack.push(ContainerNode::BulletList(Vec::new()));
            }
            Tag::List(Some(_)) => {
                self.stack.push(ContainerNode::OrderedList(Vec::new()));
            }
            Tag::Item => {
                self.stack.push(ContainerNode::ListItem(Vec::new()));
            }
            Tag::CodeBlock(kind) => {
                let lang = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                        let l = lang.to_string();
                        if l.is_empty() {
                            None
                        } else {
                            Some(l)
                        }
                    }
                    pulldown_cmark::CodeBlockKind::Indented => None,
                };
                self.stack
                    .push(ContainerNode::CodeBlock(lang, String::new()));
            }
            Tag::BlockQuote(_) => {
                self.stack.push(ContainerNode::Blockquote(Vec::new()));
            }
            Tag::Emphasis => {
                self.marks.push(Mark::Em);
            }
            Tag::Strong => {
                self.marks.push(Mark::Strong);
            }
            Tag::Link { dest_url, .. } => {
                let href = dest_url.to_string();
                self.marks.push(Mark::Link(href.clone()));
                self.link_href = Some(href);
            }
            _ => {} // Image, Table, etc.
        }
    }

    fn end_tag(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph => {
                if let Some(ContainerNode::Paragraph(content)) = self.stack.pop() {
                    let node = if content.is_empty() {
                        serde_json::json!({"type": "paragraph", "content": []})
                    } else {
                        serde_json::json!({"type": "paragraph", "content": content})
                    };
                    self.add_block_node(node);
                }
            }
            TagEnd::Heading(level) => {
                let level_num = match level {
                    pulldown_cmark::HeadingLevel::H1 => 1,
                    pulldown_cmark::HeadingLevel::H2 => 2,
                    pulldown_cmark::HeadingLevel::H3 => 3,
                    pulldown_cmark::HeadingLevel::H4 => 4,
                    pulldown_cmark::HeadingLevel::H5 => 5,
                    pulldown_cmark::HeadingLevel::H6 => 6,
                };
                if let Some(ContainerNode::Heading(content)) = self.stack.pop() {
                    let node = serde_json::json!({
                        "type": "heading",
                        "attrs": {"level": level_num},
                        "content": content
                    });
                    self.add_block_node(node);
                }
            }
            TagEnd::List(false) => {
                if let Some(ContainerNode::BulletList(items)) = self.stack.pop() {
                    let node = serde_json::json!({"type": "bulletList", "content": items});
                    self.add_block_node(node);
                }
            }
            TagEnd::List(true) => {
                if let Some(ContainerNode::OrderedList(items)) = self.stack.pop() {
                    let node = serde_json::json!({"type": "orderedList", "content": items});
                    self.add_block_node(node);
                }
            }
            TagEnd::Item => {
                if let Some(ContainerNode::ListItem(content)) = self.stack.pop() {
                    let node = serde_json::json!({"type": "listItem", "content": content});
                    // Add to parent list
                    self.add_to_list_parent(node);
                }
            }
            TagEnd::CodeBlock => {
                if let Some(ContainerNode::CodeBlock(lang, text)) = self.stack.pop() {
                    let mut node = serde_json::json!({
                        "type": "codeBlock",
                        "content": [{"type": "text", "text": text}]
                    });
                    if let Some(language) = lang {
                        node["attrs"] = serde_json::json!({"language": language});
                    }
                    self.add_block_node(node);
                }
            }
            TagEnd::BlockQuote(_) => {
                if let Some(ContainerNode::Blockquote(content)) = self.stack.pop() {
                    let node = serde_json::json!({"type": "blockquote", "content": content});
                    self.add_block_node(node);
                }
            }
            TagEnd::Emphasis => {
                self.marks.retain(|m| !matches!(m, Mark::Em));
            }
            TagEnd::Strong => {
                self.marks.retain(|m| !matches!(m, Mark::Strong));
            }
            TagEnd::Link => {
                self.marks.retain(|m| !matches!(m, Mark::Link(_)));
                self.link_href = None;
            }
            _ => {}
        }
    }

    fn add_text(&mut self, text: &str) {
        // If inside a code block, accumulate text
        if let Some(ContainerNode::CodeBlock(_, ref mut buf)) = self.stack.last_mut() {
            buf.push_str(text);
            return;
        }

        let mut node = serde_json::json!({"type": "text", "text": text});

        // Apply active marks
        let marks = self.build_marks();
        if !marks.is_empty() {
            node["marks"] = serde_json::Value::Array(marks);
        }

        self.add_node(node);
    }

    fn add_inline_code(&mut self, code: &str) {
        let node = serde_json::json!({
            "type": "text",
            "text": code,
            "marks": [{"type": "code"}]
        });
        self.add_node(node);
    }

    fn build_marks(&self) -> Vec<serde_json::Value> {
        let mut marks = Vec::new();
        for mark in &self.marks {
            match mark {
                Mark::Strong => marks.push(serde_json::json!({"type": "strong"})),
                Mark::Em => marks.push(serde_json::json!({"type": "em"})),
                Mark::Link(href) => {
                    marks.push(serde_json::json!({"type": "link", "attrs": {"href": href}}))
                }
            }
        }
        marks
    }

    /// Add an inline node to the current container (paragraph, heading, etc.)
    fn add_node(&mut self, node: serde_json::Value) {
        match self.stack.last_mut() {
            Some(ContainerNode::Paragraph(ref mut content))
            | Some(ContainerNode::Heading(ref mut content)) => {
                content.push(node);
            }
            Some(ContainerNode::ListItem(ref mut content)) => {
                // If the last item is a paragraph, add to it; otherwise wrap in paragraph
                if let Some(last) = content.last_mut() {
                    if last.get("type").and_then(|t| t.as_str()) == Some("paragraph") {
                        if let Some(arr) = last.get_mut("content").and_then(|c| c.as_array_mut()) {
                            arr.push(node);
                            return;
                        }
                    }
                }
                // No paragraph exists yet — create one
                content.push(serde_json::json!({"type": "paragraph", "content": [node]}));
            }
            Some(ContainerNode::Doc(ref mut content)) => {
                // Inline at doc level — wrap in paragraph
                content.push(serde_json::json!({"type": "paragraph", "content": [node]}));
            }
            _ => {}
        }
    }

    /// Add a block-level node (paragraph, heading, list, code block, etc.) to the current container.
    fn add_block_node(&mut self, node: serde_json::Value) {
        match self.stack.last_mut() {
            Some(ContainerNode::Doc(ref mut content))
            | Some(ContainerNode::Blockquote(ref mut content))
            | Some(ContainerNode::ListItem(ref mut content)) => {
                content.push(node);
            }
            _ => {
                // Fallback: push to bottom of stack (doc)
                if let Some(ContainerNode::Doc(ref mut content)) = self.stack.first_mut() {
                    content.push(node);
                }
            }
        }
    }

    /// Add a list item node to the parent list container.
    fn add_to_list_parent(&mut self, item: serde_json::Value) {
        match self.stack.last_mut() {
            Some(ContainerNode::BulletList(ref mut items))
            | Some(ContainerNode::OrderedList(ref mut items)) => {
                items.push(item);
            }
            _ => {
                // Shouldn't happen, but gracefully add as block node
                self.add_block_node(item);
            }
        }
    }

    fn finish(mut self) -> serde_json::Value {
        // Pop remaining stack items (shouldn't have any beyond doc)
        while self.stack.len() > 1 {
            self.stack.pop();
        }

        let content = match self.stack.pop() {
            Some(ContainerNode::Doc(content)) => content,
            _ => Vec::new(),
        };

        serde_json::json!({
            "type": "doc",
            "version": 1,
            "content": content
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_content(doc: &serde_json::Value) -> &Vec<serde_json::Value> {
        doc.get("content").unwrap().as_array().unwrap()
    }

    fn get_text(node: &serde_json::Value) -> &str {
        node.get("content")
            .and_then(|c| c.as_array())
            .and_then(|a| a.first())
            .and_then(|n| n.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("")
    }

    #[test]
    fn paragraph_text() {
        let doc = markdown_to_adf("Hello world");
        assert_eq!(doc["type"], "doc");
        assert_eq!(doc["version"], 1);
        let content = get_content(&doc);
        assert_eq!(content.len(), 1);
        assert_eq!(content[0]["type"], "paragraph");
        assert_eq!(get_text(&content[0]), "Hello world");
    }

    #[test]
    fn heading_levels_1_through_6() {
        for level in 1..=6 {
            let md = format!("{} Heading {}", "#".repeat(level), level);
            let doc = markdown_to_adf(&md);
            let content = get_content(&doc);
            assert_eq!(content[0]["type"], "heading");
            assert_eq!(content[0]["attrs"]["level"], level as u64);
        }
    }

    #[test]
    fn bold_mark() {
        let doc = markdown_to_adf("Some **bold** text");
        let content = get_content(&doc);
        let para = &content[0];
        let inlines = para["content"].as_array().unwrap();
        // Find the bold text node
        let bold_node = inlines
            .iter()
            .find(|n| n.get("text").and_then(|t| t.as_str()) == Some("bold"))
            .expect("Should have bold text node");
        let marks = bold_node["marks"].as_array().unwrap();
        assert!(marks.iter().any(|m| m["type"] == "strong"));
    }

    #[test]
    fn italic_mark() {
        let doc = markdown_to_adf("Some *italic* text");
        let content = get_content(&doc);
        let inlines = content[0]["content"].as_array().unwrap();
        let italic_node = inlines
            .iter()
            .find(|n| n.get("text").and_then(|t| t.as_str()) == Some("italic"))
            .expect("Should have italic text node");
        let marks = italic_node["marks"].as_array().unwrap();
        assert!(marks.iter().any(|m| m["type"] == "em"));
    }

    #[test]
    fn inline_code_mark() {
        let doc = markdown_to_adf("Use `println!` here");
        let content = get_content(&doc);
        let inlines = content[0]["content"].as_array().unwrap();
        let code_node = inlines
            .iter()
            .find(|n| n.get("text").and_then(|t| t.as_str()) == Some("println!"))
            .expect("Should have code text node");
        let marks = code_node["marks"].as_array().unwrap();
        assert!(marks.iter().any(|m| m["type"] == "code"));
    }

    #[test]
    fn bullet_list() {
        let doc = markdown_to_adf("- First\n- Second\n- Third");
        let content = get_content(&doc);
        assert_eq!(content[0]["type"], "bulletList");
        let items = content[0]["content"].as_array().unwrap();
        assert_eq!(items.len(), 3);
        for item in items {
            assert_eq!(item["type"], "listItem");
        }
    }

    #[test]
    fn nested_bullet_list() {
        let doc = markdown_to_adf("- Outer\n  - Inner\n- Another");
        let content = get_content(&doc);
        assert_eq!(content[0]["type"], "bulletList");
        let items = content[0]["content"].as_array().unwrap();
        // The first item should contain a nested bulletList
        let first_item_content = items[0]["content"].as_array().unwrap();
        let has_nested = first_item_content.iter().any(|n| n["type"] == "bulletList");
        assert!(has_nested, "First item should contain nested bullet list");
    }

    #[test]
    fn ordered_list() {
        let doc = markdown_to_adf("1. Alpha\n2. Beta");
        let content = get_content(&doc);
        assert_eq!(content[0]["type"], "orderedList");
        let items = content[0]["content"].as_array().unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn code_block_without_language() {
        let doc = markdown_to_adf("```\nfn main() {}\n```");
        let content = get_content(&doc);
        assert_eq!(content[0]["type"], "codeBlock");
        let text = content[0]["content"].as_array().unwrap()[0]["text"]
            .as_str()
            .unwrap();
        assert!(text.contains("fn main()"));
        // No attrs when no language
        assert!(content[0].get("attrs").is_none());
    }

    #[test]
    fn code_block_with_language() {
        let doc = markdown_to_adf("```rust\nlet x = 1;\n```");
        let content = get_content(&doc);
        assert_eq!(content[0]["type"], "codeBlock");
        assert_eq!(content[0]["attrs"]["language"], "rust");
    }

    #[test]
    fn blockquote() {
        let doc = markdown_to_adf("> Quoted text");
        let content = get_content(&doc);
        assert_eq!(content[0]["type"], "blockquote");
        let inner = content[0]["content"].as_array().unwrap();
        assert_eq!(inner[0]["type"], "paragraph");
    }

    #[test]
    fn horizontal_rule() {
        let doc = markdown_to_adf("Above\n\n---\n\nBelow");
        let content = get_content(&doc);
        let has_rule = content.iter().any(|n| n["type"] == "rule");
        assert!(has_rule, "Should contain a rule node");
    }

    #[test]
    fn links() {
        let doc = markdown_to_adf("[Click here](https://example.com)");
        let content = get_content(&doc);
        let inlines = content[0]["content"].as_array().unwrap();
        let link_node = inlines
            .iter()
            .find(|n| n.get("text").and_then(|t| t.as_str()) == Some("Click here"))
            .expect("Should have link text node");
        let marks = link_node["marks"].as_array().unwrap();
        let link_mark = marks.iter().find(|m| m["type"] == "link").unwrap();
        assert_eq!(link_mark["attrs"]["href"], "https://example.com");
    }

    #[test]
    fn mixed_content() {
        let doc = markdown_to_adf("Hello **bold** and [link](https://ex.com) done");
        let content = get_content(&doc);
        assert_eq!(content[0]["type"], "paragraph");
        let inlines = content[0]["content"].as_array().unwrap();
        assert!(inlines.len() >= 3, "Should have multiple inline nodes");
    }

    #[test]
    fn round_trip_fidelity() {
        let md = "# Title\n\nSome **bold** text.\n\n- Item one\n- Item two\n\n```\ncode here\n```";
        let adf = markdown_to_adf(md);
        let rendered = crate::adf::render_adf(&adf, false);
        assert!(rendered.contains("# Title"), "Heading preserved");
        assert!(rendered.contains("bold"), "Bold text preserved");
        assert!(rendered.contains("- Item one"), "List preserved");
        assert!(rendered.contains("code here"), "Code block preserved");
    }

    #[test]
    fn convert_body_markdown_with_description_field() {
        let body = r##"{"summary": "Bug fix", "description": "# Heading\n\nSome text"}"##;
        let result = convert_body_markdown(body).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        // summary should be unchanged (not an ADF field)
        assert_eq!(parsed["summary"], "Bug fix");
        // description should be converted to ADF object
        assert_eq!(parsed["description"]["type"], "doc");
        assert_eq!(parsed["description"]["version"], 1);
    }

    #[test]
    fn convert_body_markdown_nested_fields() {
        let body = r##"{"fields": {"description": "# Nested heading", "summary": "unchanged"}}"##;
        let result = convert_body_markdown(body).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["fields"]["description"]["type"], "doc");
        assert_eq!(parsed["fields"]["summary"], "unchanged");
    }

    #[test]
    fn convert_body_markdown_skips_object_values() {
        // If description is already an object, don't try to convert it
        let body = r#"{"description": {"type": "doc", "version": 1, "content": []}}"#;
        let result = convert_body_markdown(body).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["description"]["type"], "doc");
    }

    #[test]
    fn empty_markdown() {
        let doc = markdown_to_adf("");
        assert_eq!(doc["type"], "doc");
        let content = get_content(&doc);
        assert!(content.is_empty());
    }

    #[test]
    fn whitespace_only_markdown() {
        let doc = markdown_to_adf("   \n\n   ");
        assert_eq!(doc["type"], "doc");
        // pulldown-cmark may or may not produce content for whitespace
        // but the doc structure should be valid
        assert!(doc.get("content").unwrap().is_array());
    }

    #[test]
    fn convert_body_markdown_invalid_json() {
        let result = convert_body_markdown("not json");
        assert!(result.is_err());
    }
}
