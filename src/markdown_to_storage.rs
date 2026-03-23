//! Markdown to Confluence storage format (XHTML) converter.
//!
//! Converts Markdown input into Confluence storage format suitable for the
//! Confluence v2 API page body with `representation: "storage"`.

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

/// Convert a Markdown string to Confluence storage format (XHTML).
pub fn markdown_to_storage(markdown: &str) -> String {
    let options = Options::ENABLE_STRIKETHROUGH;
    let parser = Parser::new_ext(markdown, options);

    let mut output = String::new();
    let mut list_stack: Vec<ListKind> = Vec::new();

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => output.push_str("<p>"),
                Tag::Heading { level, .. } => {
                    output.push_str(&format!("<h{}>", level as u8));
                }
                Tag::Strong => output.push_str("<strong>"),
                Tag::Emphasis => output.push_str("<em>"),
                Tag::Strikethrough => output.push_str("<del>"),
                Tag::BlockQuote(_) => output.push_str("<blockquote>"),
                Tag::List(first_item) => {
                    if let Some(start) = first_item {
                        if start == 1 {
                            output.push_str("<ol>");
                        } else {
                            output.push_str(&format!("<ol start=\"{}\">", start));
                        }
                        list_stack.push(ListKind::Ordered);
                    } else {
                        output.push_str("<ul>");
                        list_stack.push(ListKind::Unordered);
                    }
                }
                Tag::Item => output.push_str("<li>"),
                Tag::CodeBlock(kind) => {
                    output.push_str(
                        "<ac:structured-macro ac:name=\"code\">",
                    );
                    if let CodeBlockKind::Fenced(lang) = kind {
                        let lang_str = lang.as_ref();
                        if !lang_str.is_empty() {
                            output.push_str(&format!(
                                "<ac:parameter ac:name=\"language\">{}</ac:parameter>",
                                escape_xml(lang_str)
                            ));
                        }
                    }
                    output.push_str("<ac:plain-text-body><![CDATA[");
                }
                Tag::Link { dest_url, .. } => {
                    output.push_str(&format!("<a href=\"{}\">", escape_xml(&dest_url)));
                }
                Tag::Image { dest_url, .. } => {
                    output.push_str(&format!(
                        "<ac:image><ri:url ri:value=\"{}\" /></ac:image>",
                        escape_xml(&dest_url)
                    ));
                }
                Tag::HtmlBlock => {}
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Paragraph => output.push_str("</p>"),
                TagEnd::Heading(level) => {
                    output.push_str(&format!("</h{}>", level as u8));
                }
                TagEnd::Strong => output.push_str("</strong>"),
                TagEnd::Emphasis => output.push_str("</em>"),
                TagEnd::Strikethrough => output.push_str("</del>"),
                TagEnd::BlockQuote(_) => output.push_str("</blockquote>"),
                TagEnd::List(_) => {
                    match list_stack.pop() {
                        Some(ListKind::Ordered) => output.push_str("</ol>"),
                        Some(ListKind::Unordered) => output.push_str("</ul>"),
                        None => {}
                    }
                }
                TagEnd::Item => output.push_str("</li>"),
                TagEnd::CodeBlock => {
                    output.push_str("]]></ac:plain-text-body></ac:structured-macro>");
                }
                TagEnd::Link => output.push_str("</a>"),
                TagEnd::Image => {} // handled in Start (self-closing)
                _ => {}
            },
            Event::Text(text) => {
                output.push_str(&escape_xml(&text));
            }
            Event::Code(code) => {
                output.push_str("<code>");
                output.push_str(&escape_xml(&code));
                output.push_str("</code>");
            }
            Event::SoftBreak => output.push('\n'),
            Event::HardBreak => output.push_str("<br />"),
            Event::Rule => output.push_str("<hr />"),
            Event::Html(html) => output.push_str(&html),
            _ => {}
        }
    }

    output
}

#[derive(Debug)]
enum ListKind {
    Ordered,
    Unordered,
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paragraph() {
        let result = markdown_to_storage("Hello world");
        assert_eq!(result, "<p>Hello world</p>");
    }

    #[test]
    fn headings() {
        assert_eq!(markdown_to_storage("# H1"), "<h1>H1</h1>");
        assert_eq!(markdown_to_storage("## H2"), "<h2>H2</h2>");
        assert_eq!(markdown_to_storage("### H3"), "<h3>H3</h3>");
    }

    #[test]
    fn bold_and_italic() {
        let result = markdown_to_storage("**bold** and *italic*");
        assert_eq!(result, "<p><strong>bold</strong> and <em>italic</em></p>");
    }

    #[test]
    fn inline_code() {
        let result = markdown_to_storage("Use `cargo build`");
        assert_eq!(result, "<p>Use <code>cargo build</code></p>");
    }

    #[test]
    fn bullet_list() {
        let result = markdown_to_storage("- one\n- two\n- three");
        assert_eq!(result, "<ul><li>one</li><li>two</li><li>three</li></ul>");
    }

    #[test]
    fn ordered_list() {
        let result = markdown_to_storage("1. first\n2. second");
        assert_eq!(result, "<ol><li>first</li><li>second</li></ol>");
    }

    #[test]
    fn code_block_with_language() {
        let result = markdown_to_storage("```rust\nfn main() {}\n```");
        assert!(result.contains("ac:name=\"code\""));
        assert!(result.contains("ac:name=\"language\">rust</ac:parameter>"));
        assert!(result.contains("fn main() {}"));
    }

    #[test]
    fn code_block_without_language() {
        let result = markdown_to_storage("```\nplain code\n```");
        assert!(result.contains("ac:name=\"code\""));
        assert!(!result.contains("ac:name=\"language\""));
        assert!(result.contains("plain code"));
    }

    #[test]
    fn link() {
        let result = markdown_to_storage("[click here](https://example.com)");
        assert_eq!(
            result,
            "<p><a href=\"https://example.com\">click here</a></p>"
        );
    }

    #[test]
    fn blockquote() {
        let result = markdown_to_storage("> quoted text");
        assert_eq!(result, "<blockquote><p>quoted text</p></blockquote>");
    }

    #[test]
    fn horizontal_rule() {
        let result = markdown_to_storage("above\n\n---\n\nbelow");
        assert!(result.contains("<hr />"));
    }

    #[test]
    fn xml_escaping() {
        let result = markdown_to_storage("a < b & c > d");
        assert!(result.contains("a &lt; b &amp; c &gt; d"));
    }

    #[test]
    fn mixed_content() {
        let md = "# Title\n\nSome **bold** text.\n\n- item 1\n- item 2\n\n```python\nprint('hi')\n```";
        let result = markdown_to_storage(md);
        assert!(result.contains("<h1>Title</h1>"));
        assert!(result.contains("<strong>bold</strong>"));
        assert!(result.contains("<ul>"));
        assert!(result.contains("ac:name=\"language\">python"));
        // Code block content is inside CDATA, so single quotes stay unescaped
        assert!(result.contains("print('hi')"));
    }
}
