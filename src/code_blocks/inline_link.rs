use std::fmt::Write;

use anyhow::Ok;

use super::CodeBlock;

pub struct InlineLink<'a> {
    title: &'a str,
    url: &'a str,
    image: Option<&'a String>,
}

impl<'a> InlineLink<'a> {
    pub fn new(title: &'a str, url: &'a str, image: Option<&'a String>) -> Self {
        Self { title, url, image }
    }
}

impl<'a> CodeBlock for InlineLink<'a> {
    fn render(&self) -> anyhow::Result<String> {
        let mut html = String::new();
        writeln!(
            &mut html,
            r###"<a class="inline-link"
                    href="{url}"
                    data-title="{title}"
                    data-url="{url}"
                    data-image="{image}">
                    {title}
            </a>"###,
            url = self.url,
            title = self.title,
            image = self.image.unwrap_or(&String::new())
        )?;
        Ok(html)
    }
}
