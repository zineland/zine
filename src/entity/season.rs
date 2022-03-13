use std::{fs, path::Path};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tera::Context;

use crate::Render;

use super::{article::Article, Entity};

/// The season entity config.
/// It parsed from season directory's `zine.toml`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Season {
    pub slug: String,
    pub number: u32,
    pub title: String,
    pub summary: Option<String>,
    pub cover: Option<String>,
    pub path: String,
    #[serde(rename(deserialize = "article"))]
    #[serde(default)]
    pub articles: Vec<Article>,
}

impl Entity for Season {
    fn parse(&mut self, source: &Path) -> Result<()> {
        // Representing a zine.toml file for season.
        #[derive(Debug, Deserialize)]
        struct SeasonFile {
            #[serde(rename = "article")]
            articles: Vec<Article>,
        }

        let dir = source.join(&self.path);
        let content = fs::read_to_string(&dir.join(crate::ZINE_FILE))?;
        let season_file = toml::from_str::<SeasonFile>(&content)?;
        self.articles = season_file.articles;

        self.articles.parse(&dir)?;
        Ok(())
    }

    fn render(&self, mut context: Context, dest: &Path) -> Result<()> {
        let season_dir = dest.join(&self.slug);
        context.insert("season", &self);

        // Render articles with number context.
        for (index, article) in self.articles.iter().enumerate() {
            let mut context = context.clone();
            context.insert("number", &(index + 1));
            article.render(context.clone(), &season_dir.join(article.slug()))?;
        }

        Render::render("season.jinja", &context, dest)?;
        Ok(())
    }
}
