use anyhow::Result;
use std::{fs, path::PathBuf};
use tera::Tera;

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
        for season in &mut zine.seasons {
            season.render(&mut self.tera, &self.target_dir)?;
        }
        Ok(())
    }
}
