use std::collections::HashMap;
use std::fmt::Write;

use super::CodeBlock;

pub struct HighlightBlock<'a> {
    bg_color: Option<&'a str>,
    outline_color: Option<&'a str>,
    content: &'a str,
}

impl<'a> HighlightBlock<'a> {
    pub fn new(options: &'a HashMap<&str, &str>, block: &'a str) -> Self {
        HighlightBlock {
            bg_color: options.get("bg_color").cloned(),
            outline_color: options.get("outline_color").cloned(),
            content: block,
        }
    }
}

impl<'a> CodeBlock for HighlightBlock<'a> {
    fn render(&self) -> anyhow::Result<String> {
        let mut html = String::new();
        writeln!(&mut html, r#"<div class="highlight">"#)?;
        writeln!(&mut html, r#" <div></div>"#)?;
        writeln!(&mut html, r#"</div>"#)?;
        Ok(html)
    }
}
