use anyhow::Result;
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};
use tera::{Context, Tera};

use crate::{render::Render, Zine};

pub struct Builder {
    target_dir: PathBuf,
    tera: Tera,
}

impl Builder {
    pub fn new(target_dir: &Path) -> Result<Self> {
        let target_dir = PathBuf::from(target_dir);
        if !target_dir.exists() {
            fs::create_dir_all(&target_dir)?;
        }
        let mut tera = Tera::new("templates/*.jinja")?;
        tera.register_function("featured", featured_fn);
        Ok(Builder { target_dir, tera })
    }

    /// Build the zine website from [`Zine`] config.
    pub fn build(&self, zine: Zine) -> Result<()> {
        let mut context = Context::new();
        context.insert("theme", &zine.theme);
        context.insert("site", &zine.site);
        // Render season pages.
        for season in &zine.seasons {
            season.render(&self.tera, context.clone(), &self.target_dir)?;
        }

        // Render normal pages.
        for page in &zine.pages {
            page.render(&self.tera, context.clone(), &self.target_dir.join("page"))?;
        }

        // Render home page.
        let mut seasons = zine.seasons;
        seasons.sort_unstable_by_key(|s| s.number);
        context.insert("seasons", &seasons);
        let mut buf = vec![];
        self.tera.render_to("index.jinja", &context, &mut buf)?;
        File::create(self.target_dir.join("index.html"))?.write_all(&buf)?;
        Ok(())
    }
}

// A tera function to filter featured articles.
fn featured_fn(
    map: &std::collections::HashMap<String, serde_json::Value>,
) -> tera::Result<serde_json::Value> {
    if let Some(serde_json::Value::Array(articles)) = map.get("articles") {
        Ok(serde_json::Value::Array(
            articles
                .iter()
                .filter(|article| article.get("featured") == Some(&serde_json::Value::Bool(true)))
                .cloned()
                .collect(),
        ))
    } else {
        Ok(serde_json::Value::Array(vec![]))
    }
}
