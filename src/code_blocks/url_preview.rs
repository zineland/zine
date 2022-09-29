use std::fmt::Write;

use anyhow::Result;

use super::CodeBlock;

pub(super) struct UrlPreviewBlock<'a> {
    pub url: &'a str,
    pub title: &'a str,
    pub description: &'a str,
    pub image: &'a str,
}

impl<'a> UrlPreviewBlock<'a> {
    pub(super) fn new(url: &'a str, title: &'a str, description: &'a str, image: &'a str) -> Self {
        UrlPreviewBlock {
            url,
            title,
            description,
            image,
        }
    }
}

impl<'a> CodeBlock for UrlPreviewBlock<'a> {
    fn render(&self) -> Result<String> {
        let mut html = String::new();
        writeln!(&mut html, r#"<div class="url-preview">"#)?;
        writeln!(&mut html, r#" <div>{}</div>"#, self.title)?;
        writeln!(&mut html, r#" <div>{}</div>"#, self.description)?;
        if !self.image.is_empty() {
            writeln!(&mut html, r#" <img src="{}" />"#, self.image)?;
        }
        writeln!(&mut html, r#" <a href="{url}">{url}</a>"#, url = self.url)?;
        writeln!(&mut html, r#"</div>"#)?;
        Ok(html)
    }
}

pub(super) struct UrlPreviewError<'a>(pub &'a str, pub &'a str);

impl<'a> CodeBlock for UrlPreviewError<'a> {
    fn render(&self) -> Result<String> {
        let mut html = String::new();
        writeln!(&mut html, r#"<div class="url-preview">"#)?;
        writeln!(&mut html, r#" <div></div>"#)?;
        writeln!(&mut html, r#" <div>Url preview error: {}</div>"#, self.1)?;
        writeln!(&mut html, r#" <a href="{url}">{url}</a>"#, url = self.0)?;
        writeln!(&mut html, r#"</div>"#)?;
        Ok(html)
    }
}
