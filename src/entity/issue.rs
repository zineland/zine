use std::{borrow::Cow, fs, path::Path};

use anyhow::{Context as _, Result};
use genkit::{html::Meta, markdown, Context};
use minijinja::Environment;
use rayon::{
    prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSliceMut,
};
use serde::{Deserialize, Serialize};
use time::Date;

use genkit::{current_mode, Mode};

use crate::engine;

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
    /// The optional introduction for this issue (parsed from convention intro.md file).
    #[serde(skip)]
    pub intro: Option<String>,
    cover: Option<String>,
    /// Default cover for each article in this issue.
    /// The global `default_cover` in [theme] section will be overrided.
    #[serde(skip_serializing)]
    default_cover: Option<String>,
    /// The publish date. Format like YYYY-MM-DD.
    #[serde(default)]
    #[serde(with = "genkit::helpers::serde_date::options")]
    pub pub_date: Option<Date>,
    /// Whether to publish the whole issue.
    #[serde(default)]
    publish: bool,
    /// The path of issue diretory.
    #[serde(skip_deserializing)]
    pub dir: String,
    /// Skip serialize `articles` since a single article page would
    /// contain a issue context, the `articles` is useless for the
    /// single article page.
    #[serde(skip_serializing, default)]
    #[serde(rename(deserialize = "article"))]
    articles: Vec<Article>,
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
    /// Check whether the issue need publish.
    ///
    /// The issue need publish in any of two conditions:
    /// - the publish property is true
    /// - in `zine serve` mode
    pub fn need_publish(&self) -> bool {
        self.publish || matches!(current_mode(), Mode::Serve)
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
            .filter(|article| article.featured && article.need_publish())
            .collect()
    }

    /// Get all articles need published.
    ///
    /// See [`Article::need_publish()`](super::Article::need_publish)
    pub fn articles(&self) -> Vec<&Article> {
        let issue_need_publish = self.need_publish();
        self.articles
            .iter()
            .filter(|article| issue_need_publish && article.need_publish())
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

        if self.default_cover.is_none()
            || matches!(self.default_cover.as_ref(), Some(cover) if cover.is_empty())
        {
            self.default_cover = self.cover.clone();
        }

        if let Some(default_cover) = self.default_cover.as_deref() {
            // Set default cover for articles in this issue if article has no `cover`.
            self.articles
                .iter_mut()
                .filter(|article| article.meta.has_empty_cover())
                .for_each(|article| article.meta.cover = Some(default_cover.to_owned()))
        }

        self.articles.parse(&dir)?;
        Ok(())
    }

    fn render(&self, env: &Environment, mut context: Context, dest: &Path) -> Result<()> {
        if !self.need_publish() {
            return Ok(());
        }

        let issue_dir = dest.join(&self.slug);
        context.insert("issue", &self);

        let articles = self
            .articles
            .iter()
            // Only render article which need published.
            .filter(|article| article.need_publish())
            .collect::<Vec<_>>();
        // Render articles with number context.
        articles
            .par_iter()
            .enumerate()
            .for_each(|(index, article)| {
                let mut context = context.clone();
                context.insert("siblings", &self.sibling_articles(index));
                context.insert("number", &(index + 1));

                let dest = issue_dir.clone();
                let article = (*article).clone();
                article
                    .render(env, context, &dest)
                    .expect("Render article failed.");
            });

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
        engine::render(env, "issue.jinja", context, issue_dir)?;
        Ok(())
    }
}
