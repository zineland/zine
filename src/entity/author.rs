use std::{borrow::Cow, collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};
use tera::Context;

use crate::{markdown, meta::Meta, Entity, Render};

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

#[derive(Serialize)]
pub struct AuthorList<'a> {
    authors: &'a [Author],
    article_counts: HashMap<&'a String, usize>,
}

impl<'a> AuthorList<'a> {
    pub fn new(authors: &'a [Author], article_counts: HashMap<&'a String, usize>) -> Self {
        Self {
            authors,
            article_counts,
        }
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
        Render::render("author.jinja", &context, dest.join(slug))?;
        Ok(())
    }
}

impl<'a> Entity for AuthorList<'a> {
    fn render(&self, mut context: Context, dest: &Path) -> anyhow::Result<()> {
        // TODO: open graph
        context.insert("authors", &self.authors);
        context.insert("article_counts", &self.article_counts);
        Render::render("author-list.jinja", &context, dest.join("authors"))?;
        Ok(())
    }
}
