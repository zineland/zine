use std::{fs, path::Path};

use anyhow::Result;
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};
use tera::Context;

use crate::Render;

use super::Entity;

#[derive(Serialize, Deserialize)]
pub struct Article {
    pub file: String,
    // The slug after this artcile rendered.
    // Default to file name if no slug specified.
    pub slug: Option<String>,
    pub title: String,
    pub author: Option<String>,
    pub cover: Option<String>,
    #[serde(default)]
    pub html: String,
    // TODO: deserialize to OffsetDateTime
    pub pub_date: String,
    // Wheter the article is an featured article.
    // Featured article will display in home page.
    #[serde(default)]
    pub featured: bool,
    #[serde(default)]
    pub publish: bool,
}

impl std::fmt::Debug for Article {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Article")
            .field("file", &self.file)
            .field("slug", &self.slug)
            .field("title", &self.title)
            .field("author", &self.author)
            .field("cover", &self.cover)
            .field("pub_date", &self.pub_date)
            .field("publish", &self.publish)
            .finish()
    }
}

impl Article {
    pub fn slug(&self) -> String {
        self.slug
            .as_ref()
            .cloned()
            .unwrap_or_else(|| self.file.replace(".md", ""))
    }
}

impl Entity for Article {
    fn parse(&mut self, source: &Path) -> Result<()> {
        let markdown = fs::read_to_string(&source.join(&self.file))?;
        let markdown_parser = Parser::new_ext(&markdown, Options::all());
        html::push_html(&mut self.html, markdown_parser);
        Ok(())
    }

    fn render(&self, mut context: Context, dest: &Path) -> Result<()> {
        context.insert("article", &self);
        Render::render("article.jinja", &context, dest)?;
        Ok(())
    }
}
