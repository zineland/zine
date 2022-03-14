use anyhow::Result;
use serde::Deserialize;
use std::{fs, path::Path};
use tera::Context;
use walkdir::WalkDir;

mod article;
mod end_matter;
mod page;
mod season;
mod site;
mod theme;

pub(super) use end_matter::EndMatter;

use crate::Render;

use site::Site;

use self::{page::Page, season::Season, theme::Theme};

/// The root zine entity config.
///
/// It parsed from the root directory's `zine.toml`.
#[derive(Deserialize)]
pub struct Zine {
    pub site: Site,
    pub theme: Theme,
    #[serde(default)]
    #[serde(rename = "season")]
    pub seasons: Vec<Season>,
    #[serde(rename = "page")]
    #[serde(default)]
    pub pages: Vec<Page>,
}

impl std::fmt::Debug for Zine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Zine")
            .field("site", &self.site)
            .field("theme", &self.theme)
            .field("seasons", &self.seasons)
            .finish()
    }
}

/// A trait represents the entity of zine config file.
///
/// A zine entity contains two stage:
/// - **parse**, the stage the entity to parse its attribute, such as parse markdown to html.
/// - **render**, the stage to render the entity to html file.
///
/// [`Entity`] have default empty implementations for both methods.
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

impl<T: Entity> Entity for Vec<T> {
    fn parse(&mut self, source: &Path) -> Result<()> {
        for item in self {
            item.parse(source)?;
        }
        Ok(())
    }

    fn render(&self, render: Context, dest: &Path) -> Result<()> {
        for item in self {
            item.render(render.clone(), dest)?;
        }
        Ok(())
    }
}

impl Entity for Zine {
    fn parse(&mut self, source: &Path) -> Result<()> {
        self.theme.parse(source)?;

        self.seasons.parse(source)?;
        // Sort all seasons by number.
        self.seasons.sort_unstable_by_key(|s| s.number);

        // Parse pages
        let page_dir = source.join("pages");
        for entry in WalkDir::new(&page_dir) {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let markdown = fs::read_to_string(path)?;
                self.pages.push(Page {
                    markdown,
                    file_path: path.strip_prefix(&page_dir)?.to_owned(),
                });
            }
        }
        Ok(())
    }

    fn render(&self, mut context: Context, dest: &Path) -> Result<()> {
        context.insert("theme", &self.theme);
        context.insert("site", &self.site);
        // Render all seasons pages.
        self.seasons.render(context.clone(), dest)?;

        // Render other pages.
        self.pages.render(context.clone(), &dest)?;

        // Render home page.
        context.insert("seasons", &self.seasons);
        Render::render("index.jinja", &context, dest)?;
        Ok(())
    }
}
