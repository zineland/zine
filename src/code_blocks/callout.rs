use std::collections::HashMap;
use std::fmt::Write;

use super::CodeBlock;

const DEFAULT_BG_COLOR: &str = "#f0f4ff";

pub struct CalloutBlock<'a> {
    bg_color: Option<&'a str>,
    border_color: Option<&'a str>,
    content: &'a str,
}

impl<'a> CalloutBlock<'a> {
    pub fn new(options: HashMap<&str, &'a str>, block: &'a str) -> Self {
        CalloutBlock {
            bg_color: options.get("bg_color").cloned(),
            border_color: options.get("border_color").cloned(),
            content: block,
        }
    }
}

impl<'a> CodeBlock for CalloutBlock<'a> {
    fn render(&self) -> anyhow::Result<String> {
        let mut html = String::new();
        let mut style = format!(
            "background-color: {};",
            self.bg_color.unwrap_or(DEFAULT_BG_COLOR)
        );
        if let Some(border_color) = self.border_color {
            write!(&mut style, "border-color: {}", border_color)?;
        }

        writeln!(&mut html, r#"<div class="callout" style="{}">"#, style)?;
        writeln!(&mut html, r#" <div></div>"#)?;
        writeln!(&mut html, r#"</div>"#)?;
        Ok(html)
    }
}
