use anyhow::Result;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use std::path::Path;
use tera::Context;

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
        for entity in self {
            let entity = entity.clone();
            let context = context.clone();
            let dest = dest.to_path_buf();
            tokio::task::spawn_blocking(move || {
                entity.render(context, &dest).expect("Render failed.")
            });
        }
        Ok(())
    }
}
