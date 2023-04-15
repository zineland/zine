use anyhow::Result;
use minijinja::Environment;
use rayon::{
    iter::{IntoParallelRefMutIterator, ParallelIterator},
    prelude::IntoParallelRefIterator,
};
use std::path::Path;
// use tera::Context;

mod article;
mod author;
mod issue;
mod list;
mod markdown;
mod page;
mod site;
mod theme;
mod topic;
mod zine;

use crate::context::Context;

pub use self::zine::Zine;
pub use article::{Article, MetaArticle};
pub use author::{Author, AuthorId};
pub use issue::Issue;
pub use list::List;
pub use markdown::MarkdownConfig;
pub use page::Page;
pub use site::Site;
pub use theme::Theme;
pub use topic::Topic;

/// A trait represents the entity of zine config file.
///
/// A zine entity contains two stage:
/// - **parse**, the stage the entity to parse its attribute, such as parse markdown to html.
/// - **render**, the stage to render the entity to html file.
///
/// [`Entity`] has default empty implementations for both methods.
#[allow(unused_variables)]
pub trait Entity {
    fn parse(&mut self, source: &Path) -> Result<()> {
        Ok(())
    }

    fn render(&self, context: Context, dest: &Path) -> Result<()> {
        Ok(())
    }
}

pub trait Entity2 {
    fn parse2(&mut self, source: &Path) -> Result<()>;
    fn render2(&self, env: &Environment, context: Context, dest: &Path) -> Result<()>;
}

// implement Entity2 for Option<T> and Vec<T>
impl<T: Entity2> Entity2 for Option<T> {
    fn parse2(&mut self, source: &Path) -> Result<()> {
        if let Some(entity) = self {
            entity.parse2(source)?;
        }
        Ok(())
    }

    fn render2(&self, env: &Environment, context: Context, dest: &Path) -> Result<()> {
        if let Some(entity) = self {
            entity.render2(env, context, dest)?;
        }
        Ok(())
    }
}

impl<T: Entity2 + Sync + Send + Clone + 'static> Entity2 for Vec<T> {
    fn parse2(&mut self, source: &Path) -> Result<()> {
        self.par_iter_mut()
            .try_for_each(|entity| entity.parse2(source))
    }

    fn render2(&self, env: &Environment, context: Context, dest: &Path) -> Result<()> {
        self.par_iter().try_for_each(|entity| {
            let context = context.clone();
            // let dest = dest.to_path_buf();
            entity.render2(env, context, &dest)
        })
    }
}

impl<T: Entity> Entity for Option<T> {
    fn parse(&mut self, source: &Path) -> Result<()> {
        if let Some(entity) = self {
            entity.parse(source)?;
        }
        Ok(())
    }

    fn render(&self, context: Context, dest: &Path) -> Result<()> {
        if let Some(entity) = self {
            entity.render(context, dest)?;
        }
        Ok(())
    }
}

impl<T: Entity + Sync + Send + Clone + 'static> Entity for Vec<T> {
    fn parse(&mut self, source: &Path) -> Result<()> {
        self.par_iter_mut()
            .try_for_each(|entity| entity.parse(source))
    }

    fn render(&self, context: Context, dest: &Path) -> Result<()> {
        self.par_iter().try_for_each(|entity| {
            let context = context.clone();
            entity.render(context, dest)
        })
    }
}
