use std::{borrow::Cow, path::Path};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tera::Context;

use crate::{engine, markdown, meta::Meta, Entity};

/// The author of an article. Declared in the root `zine.toml`'s **[authors]** table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    /// The author id.
    #[serde(skip_deserializing, default)]
    pub id: String,
    /// The author's name. Will fallback to capitalized id if missing.
    pub name: Option<String>,
    /// The optional avatar url. Will fallback to default zine logo if missing.
    pub avatar: Option<String>,
    /// The bio of author (markdown format).
    pub bio: String,
    /// Whether the author is an editor.
    #[serde(default)]
    #[serde(rename(deserialize = "editor"))]
    pub is_editor: bool,
}

// A [`Author`] struct with additional `article_count` field.
#[derive(Debug, Serialize)]
struct AuthorExt<'a> {
    #[serde(flatten)]
    author: &'a Author,
    // How many articles this author has.
    article_count: usize,
}

#[derive(Default, Serialize)]
pub struct AuthorList<'a> {
    authors: Vec<AuthorExt<'a>>,
}

impl<'a> AuthorList<'a> {
    pub fn record_author(&mut self, author: &'a Author, article_count: usize) {
        self.authors.push(AuthorExt {
            author,
            article_count,
        });
    }

    fn render_title(&self) -> Result<String> {
        engine::render_str(r#"{{ fluent(key="author-list") }}"#, &Context::new())
    }
}

impl Entity for Author {
    fn parse(&mut self, _source: &Path) -> anyhow::Result<()> {
        // Fallback to default zine avatar if neccessary.
        if self.avatar.is_none()
            || self.avatar.as_ref().map(|avatar| avatar.is_empty()) == Some(true)
        {
            self.avatar = Some(String::from("/static/zine.png"));
        }
        Ok(())
    }

    fn render(&self, mut context: Context, dest: &Path) -> anyhow::Result<()> {
        let slug = format!("@{}", self.id.to_lowercase());
        context.insert(
            "meta",
            &Meta {
                title: Cow::Borrowed(self.name.as_deref().unwrap_or(&self.id)),
                description: Cow::Owned(markdown::extract_description(&self.bio)),
                url: Some(Cow::Borrowed(&slug)),
                image: None,
            },
        );
        context.insert("author", &self);
        engine::render("author.jinja", &context, dest.join(slug))?;
        Ok(())
    }
}

impl<'a> Entity for AuthorList<'a> {
    fn render(&self, mut context: Context, dest: &Path) -> anyhow::Result<()> {
        context.insert(
            "meta",
            &Meta {
                title: Cow::Owned(self.render_title()?),
                description: Cow::Owned(String::new()),
                url: Some(Cow::Borrowed("authors")),
                image: None,
            },
        );
        context.insert("authors", &self.authors);
        engine::render("author-list.jinja", &context, dest.join("authors"))?;
        Ok(())
    }
}
