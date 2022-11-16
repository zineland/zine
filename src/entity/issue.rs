use std::{borrow::Cow, fs, path::Path};

use anyhow::{Context as _, Result};
use rayon::slice::ParallelSliceMut;
use serde::{Deserialize, Serialize};
use tera::Context;

use crate::{engine, html::Meta, markdown};

use super::{article::Article, Entity};

/// The issue entity config.
/// It parsed from issue directory's `zine.toml`.
#[derive(Clone, Serialize, Deserialize)]
pub struct Issue {
    /// The slug after this issue rendered.
    /// Fallback to issue path name if no slug specified.
    #[serde(default)]
    pub slug: String,
    pub number: u32,
    pub title: String,
    /// The optional introduction for this issue.
    pub intro: Option<String>,
    pub cover: Option<String>,
    /// The path of issue diretory.
    pub path: String,
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
            .field("articles", &self.articles)
            .finish()
    }
}

impl Issue {
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
            self.slug = self.path.clone();
        }

        // Parse intro file
        if let Some(intro_path) = &self.intro {
            self.intro = Some(
                fs::read_to_string(&source.join(intro_path))
                    .with_context(|| format!("Failed to read intro from {}", intro_path))?,
            );
        }

        // Representing a zine.toml file for issue.
        #[derive(Debug, Deserialize)]
        struct IssueFile {
            #[serde(rename = "article")]
            articles: Vec<Article>,
        }

        let dir = source.join(&self.path);
        let content = fs::read_to_string(&dir.join(crate::ZINE_FILE))
            .with_context(|| format!("Failed to parse `zine.toml` of `{}`", dir.display()))?;
        let issue_file = toml::from_str::<IssueFile>(&content)?;
        self.articles = issue_file.articles;
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
            let dest = issue_dir.join(article.slug());
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
        engine::render("issue.jinja", &context, issue_dir)?;
        Ok(())
    }
}
