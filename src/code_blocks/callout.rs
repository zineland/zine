use std::collections::HashMap;
use std::fmt::Write;

use crate::{data, engine::Visitor, markdown::markdown_to_html};

use super::CodeBlock;

static DEFAULT_BG_COLOR: &str = "#e1eaff";
static DEFAULT_BORDER_COLOR: &str = "#82a7fc";

/// The CalloutBlock to highlight some pragraphs.
pub struct CalloutBlock<'a> {
    bg_color: &'a str,
    border_color: &'a str,
    content: &'a str,
}

impl<'a> CalloutBlock<'a> {
    pub fn new(options: HashMap<String, &'a str>, block: &'a str) -> Self {
        let (bg_color, border_color) = Self::parse_colors(&options);
        CalloutBlock {
            bg_color,
            border_color,
            content: block,
        }
    }

    fn parse_colors(options: &HashMap<String, &'a str>) -> (&'a str, &'a str) {
        let (bg_color, border_color) = (
            options.get("bg_color").cloned(),
            options.get("border_color").cloned(),
        );

        let (theme_bg_color, theme_border_color) =
            match options.get("theme").map(|theme| theme.to_lowercase()) {
                Some(theme) => {
                    match theme.as_ref() {
                        "grey" | "gray" => ("#dee0e399", "#dee0e3"),
                        "red" => ("#fde2e2", "#f98e8b"),
                        "orange" => ("#feead2", "#ffba6b"),
                        "yellow" => ("#ffffcc", "#fff67a"),
                        "green" => ("#d9f5d6", "#8ee085"),
                        "purple" => ("#eceafe", "#ad82f7"),
                        // Default is the blue theme.
                        _ => (DEFAULT_BG_COLOR, DEFAULT_BORDER_COLOR),
                    }
                }
                None => (DEFAULT_BG_COLOR, DEFAULT_BORDER_COLOR),
            };

        (
            bg_color.unwrap_or(theme_bg_color),
            border_color.unwrap_or(theme_border_color),
        )
    }
}

impl<'a> CodeBlock for CalloutBlock<'a> {
    fn render(&self) -> anyhow::Result<String> {
        let mut html = String::new();
        let style = format!(
            "background-color: {}; border-color: {}",
            self.bg_color, self.border_color,
        );
        writeln!(&mut html, r#"<div class="callout" style="{}">"#, style)?;

        let zine_data = data::read();
        let markdown_config = zine_data.get_markdown_config();
        let block_html = markdown_to_html(self.content, Visitor::new(markdown_config));
        writeln!(&mut html, r#" <div>{}</div>"#, block_html)?;
        writeln!(&mut html, r#"</div>"#)?;
        Ok(html)
    }
}

#[cfg(test)]
mod tests {
    use crate::{code_blocks::Fenced, markdown::MarkdownVisitor};

    use super::CalloutBlock;

    struct DummyVisitor;
    impl<'a> MarkdownVisitor<'a> for DummyVisitor {}

    #[test]
    fn test_parse_colors() {
        let fenced = Fenced::parse("callout, theme: red").unwrap();
        let callout = CalloutBlock::new(fenced.options, "dummy");
        assert_eq!(callout.bg_color, "#fde2e2");
        assert_eq!(callout.border_color, "#f98e8b");

        let fenced = Fenced::parse("callout, theme: red, bg_color: #123456").unwrap();
        let callout = CalloutBlock::new(fenced.options, "dummy");
        assert_eq!(callout.bg_color, "#123456");
        assert_eq!(callout.border_color, "#f98e8b");
    }
}
