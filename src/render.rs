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

        for (index, artcile) in self.articles.iter().enumerate() {
            let mut context = context.clone();
            context.insert("number", &(index + 1));
            artcile.render(tera, context, &dir)?;
        }
        Ok(())
    }
}

impl Render for Article {
    fn render(&self, tera: &Tera, mut context: Context, path: &Path) -> Result<()> {
        let mut buf = vec![];
        context.insert("article", &self);
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
        tera.render_to("page.jinja", &context, &mut buf)?;
        let dir = path.join(&self.slug());
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        File::create(dir.join("index.html"))?.write_all(&buf)?;
        Ok(())
    }
}
