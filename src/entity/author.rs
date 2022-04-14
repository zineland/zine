use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::{markdown, meta::Meta, Entity, Render};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    #[serde(skip_deserializing, default)]
    pub id: String,
    pub name: String,
    pub avatar: Option<String>,
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
                title: Cow::Borrowed(&self.name),
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
