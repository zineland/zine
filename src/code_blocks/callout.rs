use std::collections::HashMap;
use std::fmt::Write;

use crate::engine;
use crate::markdown::markdown_to_html;

use super::CodeBlock;

/// The CalloutBlock to highlight some pragraphs.
pub struct CalloutBlock<'a> {
    bg_color: Option<&'a str>,
    border_color: Option<&'a str>,
    content: &'a str,
    visitor: engine::Vistor<'a>,
}

impl<'a> CalloutBlock<'a> {
    pub fn new(
        options: HashMap<String, &'a str>,
        block: &'a str,
        visitor: engine::Vistor<'a>,
    ) -> Self {
        CalloutBlock {
            bg_color: options.get("bg_color").cloned(),
            border_color: options.get("border_color").cloned(),
            content: block,
            visitor,
        }
    }
}

impl<'a> CodeBlock for CalloutBlock<'a> {
    fn render(&self) -> anyhow::Result<String> {
        let mut html = String::new();
        let style = format!(
            "background-color: {}; border-color: {}",
            self.bg_color.unwrap_or("#fff"),
            self.border_color.unwrap_or("var(--primary-color)"),
        );
        writeln!(&mut html, r#"<div class="callout" style="{}">"#, style)?;
        let block_html = markdown_to_html(self.content, self.visitor.clone());
        writeln!(&mut html, r#" <div>{}</div>"#, block_html)?;
        writeln!(&mut html, r#"</div>"#)?;
        Ok(html)
    }
}
