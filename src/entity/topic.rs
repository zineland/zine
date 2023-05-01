use std::{borrow::Cow, path::Path};

use anyhow::Result;
use minijinja::Environment;
use serde::{Deserialize, Serialize};

use crate::engine;

use super::Entity;
use genkit::{html::Meta, Context};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Topic {
    #[serde(skip_deserializing, default)]
    pub id: String,
    name: Option<String>,
    description: Option<String>,
}

impl Entity for Topic {
    fn parse(&mut self, _source: &Path) -> Result<()> {
        // Fallback to capitalized id if missing.
        if self.name.is_none() {
            self.name = Some(genkit::helpers::capitalize(&self.id));
        }
        Ok(())
    }

    fn render(&self, env: &Environment, mut context: Context, dest: &Path) -> Result<()> {
        context.insert(
            "meta",
            &Meta {
                title: Cow::Borrowed(self.name.as_deref().unwrap_or(&self.id)),
                description: Cow::Borrowed(self.description.as_deref().unwrap_or("")),
                url: Some(format!("/topic/{}", self.id.to_lowercase()).into()),
                image: None,
            },
        );
        context.insert("topic", &self);
        engine::render(
            env,
            "topic.jinja",
            context,
            dest.join(self.id.to_lowercase()),
        )?;
        Ok(())
    }
}
