use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser as MarkdownParser, Tag, TagEnd};

use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

pub struct MarkdownProcessor {
    syntax_set: SyntaxSet,
    options: Options,
}

impl MarkdownProcessor {
    pub fn new() -> Self {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_TASKLISTS);

        // Initialize syntax highlighting
        let syntax_set = SyntaxSet::load_defaults_newlines();

        Self {
            syntax_set,
            options,
        }
    }

    pub fn render(&self, content: &str) -> String {
        let content_without_frontmatter = if let Some(stripped) = content.strip_prefix("---") {
            if let Some(end_idx) = stripped.find("---") {
                stripped[end_idx + 3..].trim_start()
            } else {
                content
            }
        } else {
            content
        };

        let parser = MarkdownParser::new_ext(content_without_frontmatter, self.options);
        let mut events = Vec::new();
        let mut code_buffer = String::new();
        let mut in_code_block = false;
        let mut current_lang = None;

        // First pass: collect and process events
        for event in parser {
            match event {
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                    in_code_block = true;
                    current_lang = Some(lang.as_ref().to_string());
                    continue;
                }
                Event::End(TagEnd::CodeBlock) => {
                    // Add the highlighted code block as a raw HTML event
                    let highlighted = self.highlight_code(&code_buffer, current_lang.as_deref());
                    events.push(Event::Html(highlighted.into()));
                    code_buffer.clear();
                    in_code_block = false;
                    current_lang = None;
                    continue;
                }
                Event::Text(ref text) => {
                    if in_code_block {
                        code_buffer.push_str(text);
                        continue;
                    }
                }
                _ => {}
            }

            if !in_code_block {
                events.push(event);
            }
        }

        // Second pass: render all events to HTML
        let mut html_output = String::new();
        html::push_html(&mut html_output, events.into_iter());
        html_output
    }

    fn highlight_code(&self, code: &str, language: Option<&str>) -> String {
        let syntax = match language {
            Some(lang) => self
                .syntax_set
                .find_syntax_by_token(lang)
                .or_else(|| self.syntax_set.find_syntax_by_extension(lang)),
            None => None,
        }
        .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let mut html_generator = ClassedHTMLGenerator::new_with_class_style(
            syntax,
            &self.syntax_set,
            ClassStyle::Spaced,
        );

        for line in LinesWithEndings::from(code) {
            let _ = html_generator.parse_html_for_line_which_includes_newline(line);
        }

        format!(
            "<pre class=\"highlight\"><code class=\"language-{}\">{}</code></pre>",
            language.unwrap_or("text"),
            html_generator.finalize()
        )
    }
}

impl Default for MarkdownProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_markdown() {
        let processor = MarkdownProcessor::new();
        let input = "# Hello\n\nThis is a test";
        let output = processor.render(input);
        assert!(output.contains("<h1>Hello</h1>"));
        assert!(output.contains("<p>This is a test</p>"));
    }

    #[test]
    fn test_code_highlighting() {
        let processor = MarkdownProcessor::new();
        let input = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";
        let output = processor.render(input);
        assert!(output.contains("highlight"));
        assert!(output.contains("language-rust"));
        assert!(output.contains("println!"));
    }

    #[test]
    fn test_mixed_content() {
        let processor = MarkdownProcessor::new();
        let input = r#"# Test

Some text

```python
def hello():
    print("Hello")
```

More text"#;
        let output = processor.render(input);
        assert!(output.contains("<h1>"));
        assert!(output.contains("language-python"));
        assert!(output.contains("<p>Some text</p>"));
        assert!(output.contains("<p>More text</p>"));
    }

    #[test]
    fn test_unknown_language() {
        let processor = MarkdownProcessor::new();
        let input = "```unknown-lang\nsome code\n```";
        let output = processor.render(input);
        assert!(output.contains("language-unknown-lang"));
        assert!(output.contains("some code"));
    }

    #[test]
    fn test_inline_code() {
        let processor = MarkdownProcessor::new();
        let input = "This is `inline code`";
        let output = processor.render(input);
        assert!(output.contains("<code>inline code</code>"));
    }
}
