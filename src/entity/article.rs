use std::{borrow::Cow, fs, path::Path};

use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use tera::Context;
use time::Date;

use crate::{
    current_mode, data, engine,
    html::Meta,
    markdown::{self, MarkdownRender},
    Mode,
};

use super::{AuthorId, Entity};

/// The Meta info of Article.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaArticle {
    pub file: String,
    /// The slug after this artcile rendered.
    /// Fallback to file name if no slug specified.
    #[serde(default)]
    pub slug: String,
    /// Absolute path of this article.
    /// The field take precedence over `slug` field.
    pub path: Option<String>,
    pub title: String,
    /// The author id of this article.
    /// An article can has zero, one or multiple authors.
    pub author: Option<AuthorId>,
    pub cover: Option<String>,
    /// The publish date. Format like YYYY-MM-dd.
    #[serde(with = "crate::helpers::serde_date")]
    pub pub_date: Date,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Article {
    #[serde(flatten)]
    pub meta: MetaArticle,
    /// The article's markdown content.
    #[serde(default, skip_serializing)]
    pub markdown: String,
    /// Whether the article is an featured article.
    /// Featured article will display in home page.
    #[serde(default)]
    pub featured: bool,
    /// Whether publish the article. Publish means generate the article HTML file.
    /// This field would be ignored if in `zine serve` mode, that's mean we alwasy
    /// generate HTML file in this mode.
    #[serde(default)]
    pub publish: bool,
}

impl std::fmt::Debug for Article {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Article")
            .field("meta", &self.meta)
            .field("publish", &self.publish)
            .finish()
    }
}

impl Article {
    /// Check whether `author` name is the author of this article.
    pub fn is_author(&self, author: &str) -> bool {
        self.meta
            .author
            .as_ref()
            .map(|inner| inner.is_author(author))
            .unwrap_or_default()
    }

    /// Check whether the article need publish.
    ///
    /// The article need publish in any of two conditions:
    /// - the publish property is true
    /// - in `zine serve` mode
    pub fn need_publish(&self) -> bool {
        self.publish || matches!(current_mode(), Mode::Serve)
    }

    pub fn slug(&self) -> &String {
        &self.meta.slug
    }
}

impl Entity for Article {
    fn parse(&mut self, source: &Path) -> Result<()> {
        let file_path = source.join(&self.meta.file);
        self.markdown = fs::read_to_string(&file_path).with_context(|| {
            format!("Failed to read markdown file of `{}`", file_path.display())
        })?;

        // Fallback to file name if no slug specified.
        if self.meta.path.is_none() && self.meta.slug.is_empty() {
            self.meta.slug = self.meta.file.replace(".md", "")
        }
        // Fallback to the default placeholder image if the cover is missing.
        if self.meta.cover.is_none() || matches!(&self.meta.cover, Some(cover) if cover.is_empty())
        {
            let data = data::read();
            self.meta.cover = data.get_theme().default_cover.clone();
        }
        Ok(())
    }

    fn render(&self, mut context: Context, dest: &Path) -> Result<()> {
        context.insert(
            "meta",
            &Meta {
                title: Cow::Borrowed(&self.meta.title),
                description: Cow::Owned(markdown::extract_description(&self.markdown)),
                url: Some(Cow::Borrowed(self.slug())),
                image: self.meta.cover.as_deref().map(Cow::Borrowed),
            },
        );
        context.insert("page_type", "article");
        context.insert("article", &self);

        let zine_data = data::read();
        let markdown_config = zine_data.get_markdown_config();
        let mut markdown_render = MarkdownRender::new(markdown_config);
        let html = markdown_render.render_html(&self.markdown);
        markdown_render.rebuild_toc_depth();
        context.insert("html", &html);
        context.insert("toc", &markdown_render.toc);
        drop(zine_data);

        if let Some(path) = self.meta.path.as_ref() {
            let mut dest = dest.to_path_buf();
            dest.pop();
            engine::render(
                "article.jinja",
                &context,
                dest.join(path.trim_start_matches('/')),
            )?;
        } else {
            engine::render("article.jinja", &context, dest.join(self.slug()))?;
        }
        Ok(())
    }
}
