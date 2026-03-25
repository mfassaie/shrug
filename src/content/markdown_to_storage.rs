//! Markdown to Confluence storage format (XHTML) converter.

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

/// Convert markdown to Confluence storage format (XHTML).
pub fn markdown_to_storage(markdown: &str) -> String {
    let options = Options::ENABLE_STRIKETHROUGH;
    let parser = Parser::new_ext(markdown, options);
    let mut output = String::new();

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    output.push_str(&format!("<h{}>", level as u8))
                }
                Tag::Paragraph => output.push_str("<p>"),
                Tag::Strong => output.push_str("<strong>"),
                Tag::Emphasis => output.push_str("<em>"),
                Tag::Strikethrough => output.push_str("<del>"),
                Tag::Link { dest_url, .. } => {
                    output.push_str(&format!("<a href=\"{}\">", dest_url))
                }
                Tag::List(Some(_)) => output.push_str("<ol>"),
                Tag::List(None) => output.push_str("<ul>"),
                Tag::Item => output.push_str("<li>"),
                Tag::CodeBlock(_) => output.push_str("<pre><code>"),
                Tag::BlockQuote(_) => output.push_str("<blockquote>"),
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Heading(level) => {
                    output.push_str(&format!("</h{}>", level as u8))
                }
                TagEnd::Paragraph => output.push_str("</p>"),
                TagEnd::Strong => output.push_str("</strong>"),
                TagEnd::Emphasis => output.push_str("</em>"),
                TagEnd::Strikethrough => output.push_str("</del>"),
                TagEnd::Link => output.push_str("</a>"),
                TagEnd::List(true) => output.push_str("</ol>"),
                TagEnd::List(false) => output.push_str("</ul>"),
                TagEnd::Item => output.push_str("</li>"),
                TagEnd::CodeBlock => output.push_str("</code></pre>"),
                TagEnd::BlockQuote(_) => output.push_str("</blockquote>"),
                _ => {}
            },
            Event::Text(text) => {
                // Escape HTML entities
                for ch in text.chars() {
                    match ch {
                        '&' => output.push_str("&amp;"),
                        '<' => output.push_str("&lt;"),
                        '>' => output.push_str("&gt;"),
                        '"' => output.push_str("&quot;"),
                        _ => output.push(ch),
                    }
                }
            }
            Event::Code(code) => {
                output.push_str("<code>");
                output.push_str(
                    &code
                        .replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;"),
                );
                output.push_str("</code>");
            }
            Event::SoftBreak => output.push('\n'),
            Event::HardBreak => output.push_str("<br />"),
            Event::Rule => output.push_str("<hr />"),
            _ => {}
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_to_storage_heading() {
        let result = markdown_to_storage("# Hello");
        assert!(result.contains("<h1>Hello</h1>"));
    }

    #[test]
    fn test_markdown_to_storage_paragraph() {
        let result = markdown_to_storage("Hello world");
        assert!(result.contains("<p>Hello world</p>"));
    }

    #[test]
    fn test_markdown_to_storage_bold_italic() {
        let result = markdown_to_storage("**bold** *italic*");
        assert!(result.contains("<strong>bold</strong>"));
        assert!(result.contains("<em>italic</em>"));
    }

    #[test]
    fn test_markdown_to_storage_list() {
        let result = markdown_to_storage("- item 1\n- item 2");
        assert!(result.contains("<ul>"));
        assert!(result.contains("<li>"));
        assert!(result.contains("</ul>"));
    }

    #[test]
    fn test_markdown_to_storage_ordered_list() {
        let result = markdown_to_storage("1. first\n2. second");
        assert!(result.contains("<ol>"));
        assert!(result.contains("<li>"));
        assert!(result.contains("</ol>"));
    }

    #[test]
    fn test_markdown_to_storage_code_block() {
        let result = markdown_to_storage("```\nfn main() {}\n```");
        assert!(result.contains("<pre><code>"));
        assert!(result.contains("</code></pre>"));
    }

    #[test]
    fn test_markdown_to_storage_inline_code() {
        let result = markdown_to_storage("Use `println!` here");
        assert!(result.contains("<code>println!</code>"));
    }

    #[test]
    fn test_markdown_to_storage_link() {
        let result = markdown_to_storage("[Click](https://example.com)");
        assert!(result.contains("<a href=\"https://example.com\">Click</a>"));
    }

    #[test]
    fn test_markdown_to_storage_blockquote() {
        let result = markdown_to_storage("> Quoted text");
        assert!(result.contains("<blockquote>"));
        assert!(result.contains("</blockquote>"));
    }

    #[test]
    fn test_markdown_to_storage_html_escaping() {
        let result = markdown_to_storage("Use 5 > 3 & 2 < 4");
        assert!(result.contains("&gt;"));
        assert!(result.contains("&amp;"));
        assert!(result.contains("&lt;"));
    }

    #[test]
    fn test_markdown_to_storage_horizontal_rule() {
        let result = markdown_to_storage("Above\n\n---\n\nBelow");
        assert!(result.contains("<hr />"));
    }

    #[test]
    fn test_markdown_to_storage_strikethrough() {
        let result = markdown_to_storage("~~deleted~~");
        assert!(result.contains("<del>deleted</del>"));
    }

    #[test]
    fn test_markdown_to_storage_empty() {
        let result = markdown_to_storage("");
        assert!(result.is_empty());
    }
}
