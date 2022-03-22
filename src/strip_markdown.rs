// A fork from https://github.com/arranf/strip-markdown

use pulldown_cmark::Event::{Code, End, HardBreak, Rule, SoftBreak, Start, Text};
use pulldown_cmark::{Options, Parser, Tag};

/// Convert markdown into plain text.
#[must_use]
pub fn strip_markdown(markdown: &str) -> String {
    // GFM tables and tasks lists are not enabled.
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(markdown, options);
    let mut buffer = String::new();

    // For each event we push into the buffer to produce the 'stripped' version.
    for event in parser {
        match event {
            // The start and end events don't contain the text inside the tag.
            // That's handled by the `Event::Text` arm.
            Start(tag) => start_tag(&tag, &mut buffer),
            End(tag) => end_tag(&tag, &mut buffer),
            Text(text) => {
                // FIXME: img alt text shouldn't be treated as a text?
                buffer.push_str(&text);
            }
            Code(code) => buffer.push_str(&code),
            SoftBreak | HardBreak | Rule => fresh_line(&mut buffer),
            _ => (),
        }
    }
    buffer
}

fn start_tag(tag: &Tag, buffer: &mut String) {
    match tag {
        Tag::CodeBlock(_) | Tag::List(_) => fresh_line(buffer),
        Tag::Link(_, _, title) => {
            if !title.is_empty() {
                buffer.push_str(title);
            }
        }
        _ => (),
    }
}

fn end_tag(tag: &Tag, buffer: &mut String) {
    match tag {
        Tag::Table(_)
        | Tag::TableHead
        | Tag::TableRow
        | Tag::Heading(..)
        | Tag::BlockQuote
        | Tag::CodeBlock(_)
        | Tag::Item => fresh_line(buffer),
        _ => (),
    }
}

fn fresh_line(buffer: &mut String) {
    buffer.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_inline_strong() {
        let markdown = r#"**Hello**"#;
        let expected = "Hello";
        assert_eq!(strip_markdown(markdown), expected);
    }

    #[test]
    fn basic_inline_emphasis() {
        let markdown = r#"_Hello_"#;
        let expected = "Hello";
        assert_eq!(strip_markdown(markdown), expected);
    }

    #[test]
    fn basic_header() {
        let markdown = r#"# Header"#;
        let expected = "Header\n";
        assert_eq!(strip_markdown(markdown), expected);
    }

    #[test]
    fn alt_header() {
        let markdown = r#"
Header
======
"#;
        let expected = "Header\n";
        assert_eq!(strip_markdown(markdown), expected);
    }

    #[test]
    fn strong_emphasis() {
        let markdown = r#"**asterisks and _underscores_**"#;
        let expected = "asterisks and underscores";
        assert_eq!(strip_markdown(markdown), expected);
    }

    #[test]
    fn strikethrough() {
        let markdown = r#"~~strikethrough~~"#;
        let expected = "strikethrough";
        assert_eq!(strip_markdown(markdown), expected);
    }

    #[test]
    fn mixed_list() {
        let markdown = r#"
1. First ordered list item
2. Another item
1. Actual numbers don't matter, just that it's a number
  1. Ordered sub-list
4. And another item.
"#;

        let expected = r#"
First ordered list item
Another item
Actual numbers don't matter, just that it's a number
Ordered sub-list
And another item.
"#;
        assert_eq!(strip_markdown(markdown), expected);
    }

    #[test]
    fn basic_list() {
        let markdown = r#"
* alpha
* beta
"#;
        let expected = r#"
alpha
beta
"#;
        assert_eq!(strip_markdown(markdown), expected);
    }

    #[test]
    fn list_with_header() {
        let markdown = r#"# Title
* alpha
* beta
"#;
        let expected = r#"Title

alpha
beta
"#;
        assert_eq!(strip_markdown(markdown), expected);
    }

    #[test]
    fn basic_link() {
        let markdown = "[I'm an inline-style link](https://www.google.com)";
        let expected = "I'm an inline-style link";
        assert_eq!(strip_markdown(markdown), expected)
    }

    #[ignore]
    #[test]
    fn link_with_itself() {
        let markdown = "[https://www.google.com]";
        let expected = "https://www.google.com";
        assert_eq!(strip_markdown(markdown), expected)
    }

    #[test]
    fn basic_image() {
        let markdown = "![alt text](https://github.com/adam-p/markdown-here/raw/master/src/common/images/icon48.png)";
        let expected = "alt text";
        assert_eq!(strip_markdown(markdown), expected);
    }

    #[test]
    fn inline_code() {
        let markdown = "`inline code`";
        let expected = "inline code";
        assert_eq!(strip_markdown(markdown), expected);
    }

    #[test]
    fn code_block() {
        let markdown = r#"
```javascript
var s = "JavaScript syntax highlighting";
alert(s);
```"#;
        let expected = r#"
var s = "JavaScript syntax highlighting";
alert(s);

"#;
        assert_eq!(strip_markdown(markdown), expected);
    }

    #[test]
    fn block_quote() {
        let markdown = r#"> Blockquotes are very handy in email to emulate reply text.
> This line is part of the same quote."#;
        let expected = "Blockquotes are very handy in email to emulate reply text.
This line is part of the same quote.\n";
        assert_eq!(strip_markdown(markdown), expected);
    }
}
