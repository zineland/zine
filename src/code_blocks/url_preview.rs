use std::fmt::Write;

use super::CodeBlock;

pub(super) struct UrlPreviewBlock<'a>(pub &'a str, pub &'a str, pub &'a str);

impl<'a> CodeBlock for UrlPreviewBlock<'a> {
    fn render(&self) -> anyhow::Result<String> {
        let mut html = String::new();
        writeln!(&mut html, r#"<div class="url-preview">"#)?;
        writeln!(&mut html, r#" <div>{}</div>"#, self.1)?;
        writeln!(&mut html, r#" <div>{}</div>"#, self.2)?;
        writeln!(&mut html, r#" <a href="{url}">{url}</a>"#, url = self.0)?;
        writeln!(&mut html, r#"</div>"#)?;
        Ok(html)
    }
}

pub struct UrlPreviewError<'a>(pub &'a str, pub &'a str);

impl<'a> CodeBlock for UrlPreviewError<'a> {
    fn render(&self) -> anyhow::Result<String> {
        let mut html = String::new();
        writeln!(&mut html, r#"<div class="url-preview">"#)?;
        writeln!(&mut html, r#" <div></div>"#)?;
        writeln!(&mut html, r#" <div>Url preview error: {}</div>"#, self.1)?;
        writeln!(&mut html, r#" <a href="{url}">{url}</a>"#, url = self.0)?;
        writeln!(&mut html, r#"</div>"#)?;
        Ok(html)
    }
}
