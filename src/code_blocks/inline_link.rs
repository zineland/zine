use anyhow::Ok;
use tera::Context;

use crate::engine;

use super::CodeBlock;

pub struct InlineLink<'a> {
    title: &'a str,
    url: &'a str,
    image: &'a Option<String>,
}

impl<'a> InlineLink<'a> {
    pub fn new(title: &'a str, url: &'a str, image: &'a Option<String>) -> Self {
        Self { title, url, image }
    }
}

impl<'a> CodeBlock for InlineLink<'a> {
    fn render(&self) -> anyhow::Result<String> {
        let mut context = Context::new();
        context.insert("title", &self.title);
        context.insert("url", &self.url);
        context.insert("image", &self.image);
        let html = engine::get_tera().render("inline-link.jinja", &context)?;
        Ok(html)
    }
}
