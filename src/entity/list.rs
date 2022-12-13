use std::{borrow::Cow, path::Path};

use anyhow::Result;
use serde::Serialize;
use tera::Context;

use crate::{engine, html::Meta, Entity};

use super::{Author, Topic};

#[derive(Serialize)]
pub struct List<'a, E> {
    entities: Vec<EntityExt<'a, E>>,
    name: &'static str,
    template: &'static str,
    fluent_key: &'static str,
}

/// A [`Entity`] struct with additional `article_count` field.
#[derive(Serialize)]
pub(super) struct EntityExt<'a, E> {
    #[serde(flatten)]
    entity: &'a E,
    // How many articles this entity has.
    article_count: usize,
}

impl<'a, E> List<'a, E> {
    fn render_title(&self) -> Result<String> {
        engine::render_str(
            &format!(r#"{{ fluent(key="{}") }}"#, self.fluent_key),
            &Context::new(),
        )
    }
}

impl<'a> List<'a, Author> {
    pub(super) fn author_list() -> Self {
        List {
            entities: Default::default(),
            name: "authors",
            template: "author-list.jinja",
            fluent_key: "author-list",
        }
    }

    pub(super) fn push_author(&mut self, author: &'a Author, article_count: usize) {
        self.entities.push(EntityExt {
            entity: author,
            article_count,
        });
    }
}

impl<'a> List<'a, Topic> {
    pub(super) fn topic_list() -> Self {
        List {
            entities: Default::default(),
            name: "topics",
            template: "topic-list.jinja",
            fluent_key: "topic-list",
        }
    }

    pub(super) fn push_topic(&mut self, topic: &'a Topic, article_count: usize) {
        self.entities.push(EntityExt {
            entity: topic,
            article_count,
        });
    }
}

impl<'a, E: Serialize> Entity for List<'a, E> {
    fn render(&self, mut context: Context, dest: &Path) -> anyhow::Result<()> {
        context.insert(
            "meta",
            &Meta {
                title: Cow::Owned(self.render_title()?),
                description: Cow::Owned(String::new()),
                url: Some(self.name.into()),
                image: None,
            },
        );
        context.insert(self.name, &self.entities);
        engine::render(self.template, &context, dest.join(self.name))?;
        Ok(())
    }
}
