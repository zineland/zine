use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use anyhow::Result;
use tera::{Context, Tera};

use crate::{Article, Page, Season};

pub trait Render {
    fn render(&self, tera: &Tera, context: Context, path: &Path) -> Result<()>;
}

impl Render for Season {
    fn render(&self, tera: &Tera, mut context: Context, path: &Path) -> Result<()> {
        let mut buf = vec![];
        context.insert("season", &self);
        tera.render_to("season.jinja", &context, &mut buf)?;
        let dir = path.join(&self.slug);
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        File::create(dir.join("index.html"))?.write_all(&buf)?;

        for artcile in &self.articles {
            artcile.render(tera, context.clone(), &dir)?;
        }
        Ok(())
    }
}

impl Render for Article {
    fn render(&self, tera: &Tera, mut context: Context, path: &Path) -> Result<()> {
        let mut buf = vec![];
        context.insert("content", &self.html);
        tera.render_to("article.jinja", &context, &mut buf)?;
        let dir = path.join(&self.slug());
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        File::create(dir.join("index.html"))?.write_all(&buf)?;
        Ok(())
    }
}

impl Render for Page {
    fn render(&self, tera: &Tera, mut context: Context, path: &Path) -> Result<()> {
        let mut buf = vec![];
        context.insert("content", &self.html);
        tera.render_to("article.jinja", &context, &mut buf)?;
        let dir = path.join(&self.slug());
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        File::create(dir.join("index.html"))?.write_all(&buf)?;
        Ok(())
    }
}
