use anyhow::Result;
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};
use tera::{Context, Tera};

use crate::{render::Render, Zine};

pub struct Builder {
    target_dir: PathBuf,
    tera: Tera,
}

impl Builder {
    pub fn new(target_dir: &str) -> Result<Self> {
        let target_dir = PathBuf::from(target_dir);
        if target_dir.exists() {
            fs::remove_dir_all(&target_dir)?;
        } else {
            fs::create_dir_all(&target_dir)?;
        }
        let tera = Tera::new("templates/*.jinja")?;
        Ok(Builder { target_dir, tera })
    }

    /// Build the zine website from [`Zine`] config.
    pub fn build(&mut self, mut zine: Zine) -> Result<()> {
        let mut context = Context::new();
        context.insert("site", &zine.site);
        for season in &mut zine.seasons {
            season.render(&mut self.tera, context.clone(), &self.target_dir)?;
        }

        // Render home page.
        context.insert("seasons", &zine.seasons);
        let mut buf = vec![];
        self.tera.render_to("index.jinja", &context, &mut buf)?;
        File::create(self.target_dir.join("index.html"))?.write_all(&buf)?;
        Ok(())
    }
}
