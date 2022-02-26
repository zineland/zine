use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use anyhow::Result;
use tera::{Context, Tera};

use crate::{Article, Season};

pub trait Render {
    fn render(&mut self, tera: &mut Tera, path: &Path) -> Result<()>;
}

impl Render for Season {
    fn render(&mut self, tera: &mut Tera, path: &Path) -> Result<()> {
        let mut buf = vec![];
        let context = Context::new();
        tera.render_to("season.jinja", &context, &mut buf)?;
        let dir = path.join(&self.slug);
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        File::create(dir.join("index.html"))?.write_all(&buf)?;

        for artcile in &mut self.articles {
            artcile.render(tera, &dir)?;
        }
        Ok(())
    }
}

impl Render for Article {
    fn render(&mut self, tera: &mut Tera, path: &Path) -> Result<()> {
        let mut buf = vec![];
        let context = Context::new();
        tera.render_to("article.jinja", &context, &mut buf)?;
        let dir = path.join(&self.slug());
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        File::create(dir.join("index.html"))?.write_all(&buf)?;
        Ok(())
    }
}
