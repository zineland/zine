use std::borrow::Cow;

use serde::{Deserialize, Serialize};

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
}

impl Entity for Author {
    fn parse(&mut self, _source: &std::path::Path) -> anyhow::Result<()> {
        // Fallback to default zine avatar if neccessary.
        if self.avatar.is_none()
            || self.avatar.as_ref().map(|avatar| avatar.is_empty()) == Some(true)
        {
            self.avatar = Some(String::from("/static/zine.png"));
        }
        Ok(())
    }

    fn render(&self, mut context: tera::Context, dest: &std::path::Path) -> anyhow::Result<()> {
        let slug = format!("@{}", self.id.to_lowercase());
        context.insert(
            "meta",
            &Meta {
                title: Cow::Borrowed(&self.name.as_deref().unwrap_or(&self.id)),
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
