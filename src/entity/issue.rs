use std::io::prelude::*;
use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
};

use crate::ZINE_FILE;

use anyhow::{Context as _, Result};
use rayon::slice::ParallelSliceMut;
use serde::{Deserialize, Serialize};
use tera::Context;

use crate::{engine, html::Meta, markdown};

use super::{article::Article, Entity};

/// The issue entity config.
/// It parsed from issue directory's `zine.toml`.
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Issue {
    /// The slug after this issue rendered.
    /// Fallback to issue path name if no slug specified.
    #[serde(default)]
    pub slug: String,
    pub number: u32,
    pub title: String,
    /// The optional introduction for this issue (parsed from convention intro.md file).
    #[serde(skip)]
    pub intro: Option<String>,
    pub cover: Option<String>,
    /// The path of issue diretory.
    #[serde(skip_deserializing)]
    pub dir: String,
    /// Skip serialize `articles` since a single article page would
    /// contain a issue context, the `articles` is useless for the
    /// single article page.
    #[serde(skip_serializing, default)]
    #[serde(rename(deserialize = "article"))]
    pub articles: Vec<Article>,
}

impl std::fmt::Debug for Issue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Issue")
            .field("slug", &self.slug)
            .field("number", &self.number)
            .field("title", &self.title)
            .field("intro", &self.intro.is_some())
            .field("cover", &self.cover)
            .field("dir", &self.dir)
            .field("articles", &self.articles)
            .finish()
    }
}

impl Issue {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    fn set_issue_number(&mut self, number: u32) -> &mut Self {
        self.number = number;
        self
    }
    fn set_title(&mut self, title: impl Into<String>) -> &mut Self {
        self.title = title.into();
        self.dir = self.title.clone().to_lowercase().replace(" ", "-");
        self.slug = self.dir.clone();
        self
    }
    fn set_intro(&mut self, intro: impl Into<String>) -> &mut Self {
        self.intro = Some(intro.into());
        self
    }
    // Appends the issue to the top level zine.toml file
    fn write_new_issue(&self, path: &PathBuf) -> Result<()> {
        if path.join(ZINE_FILE).exists() {
            Err(anyhow::anyhow!("Issue already Exists"))?
        }
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&path.join(ZINE_FILE))?;

        let toml_str = toml::to_string(&self)?;

        file.write_all(&toml_str.as_bytes())?;

        Ok(())
    }
    // Get the description of this issue.
    // Mainly for html meta description tag.
    fn description(&self) -> String {
        if let Some(intro) = self.intro.as_ref() {
            markdown::extract_description(intro)
        } else {
            String::default()
        }
    }

    fn sibling_articles(&self, current: usize) -> (Option<&Article>, Option<&Article>) {
        if current == 0 {
            return (None, self.articles.get(current + 1));
        }

        (
            self.articles.get(current - 1),
            self.articles.get(current + 1),
        )
    }

    pub fn featured_articles(&self) -> Vec<&Article> {
        self.articles
            .iter()
            .filter(|article| article.featured && article.publish)
            .collect()
    }
}

impl Entity for Issue {
    fn parse(&mut self, source: &Path) -> Result<()> {
        // Fallback to path if no slug specified.
        if self.slug.is_empty() {
            self.slug = self.dir.clone();
        }

        let dir = source.join(crate::ZINE_CONTENT_DIR).join(&self.dir);
        // Parse intro file
        let intro_path = dir.join(crate::ZINE_INTRO_FILE);
        if intro_path.exists() {
            self.intro =
                Some(fs::read_to_string(&intro_path).with_context(|| {
                    format!("Failed to read intro from {}", intro_path.display())
                })?);
        }

        // Sort all articles by pub_date.
        self.articles
            .par_sort_unstable_by_key(|article| article.meta.pub_date);

        self.articles.parse(&dir)?;
        Ok(())
    }

    fn render(&self, mut context: Context, dest: &Path) -> Result<()> {
        let issue_dir = dest.join(&self.slug);
        context.insert("issue", &self);

        let articles = self
            .articles
            .iter()
            // Only render article which need published.
            .filter(|article| article.need_publish())
            .collect::<Vec<_>>();
        // Render articles with number context.
        for (index, article) in articles.iter().enumerate() {
            let mut context = context.clone();
            context.insert("siblings", &self.sibling_articles(index));
            context.insert("number", &(index + 1));

            let dest = issue_dir.clone();
            let article = (*article).clone();
            tokio::task::spawn_blocking(move || {
                article
                    .render(context, &dest)
                    .expect("Render article failed.");
            });
        }

        context.insert("articles", &articles);
        context.insert(
            "meta",
            &Meta {
                title: Cow::Borrowed(&self.title),
                description: Cow::Owned(self.description()),
                url: Some(Cow::Borrowed(&self.slug)),
                image: self.cover.as_deref().map(Cow::Borrowed),
            },
        );
        context.insert("intro", &self.intro);
        engine::render("issue.jinja", &context, issue_dir)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::entity::issue::Issue;

    use std::env;

    #[test]
    fn defaults() {
        let mut issue = Issue::new();
        issue
            .set_issue_number(1)
            .set_title("Some Magical Title")
            .set_intro("Some magical introduction to some amazing Issue");

        let work_space = std::path::Path::new("/tmp");
        let path = work_space.to_path_buf();
        assert!(env::set_current_dir(&work_space).is_ok());
        assert!(issue.write_new_issue(&path).is_ok());
        assert!(issue.write_new_issue(&path).is_err());
    }
}
