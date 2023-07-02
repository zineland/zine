use std::{borrow::Cow, collections::HashMap, fs, path::Path};

use anyhow::{ensure, Context as _, Result};
use genkit::{current_mode, Mode};
use genkit::{html::Meta, markdown, Context};
use minijinja::Environment;
use rayon::prelude::{ParallelBridge, ParallelIterator};
use serde::{Deserialize, Serialize};
use time::Date;

use crate::{data, engine, i18n};

use super::{AuthorId, Entity};

/// The Meta info of Article.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaArticle {
    pub file: String,
    /// The slug after this artcile rendered.
    /// Fallback to file name if no slug specified.
    #[serde(default)]
    pub slug: String,
    /// Absolute path of this article.
    /// The field take precedence over `slug` field.
    pub path: Option<String>,
    pub title: String,
    /// The author id of this article.
    /// An article can has zero, one or multiple authors.
    pub author: Option<AuthorId>,
    pub cover: Option<String>,
    /// The publish date. Format like YYYY-MM-DD.
    #[serde(with = "genkit::helpers::serde_date")]
    #[serde(default = "MetaArticle::default_pub_date")]
    pub pub_date: Date,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Article {
    #[serde(flatten)]
    pub meta: MetaArticle,
    /// The article's markdown content.
    #[serde(default, skip_serializing)]
    pub markdown: String,
    /// The optional topics of this article.
    #[serde(default)]
    #[serde(rename(deserialize = "topic"))]
    pub topics: Vec<String>,
    /// Whether the article is an featured article.
    /// Featured article will display in home page.
    #[serde(default, skip_serializing)]
    pub featured: bool,
    /// Whether publish the article. Publish means generate the article HTML file.
    /// This field would be ignored if in `zine serve` mode, that's mean we alwasy
    /// generate HTML file in this mode.
    #[serde(default)]
    publish: bool,
    /// The canonical link of this article.
    /// See issue: https://github.com/zineland/zine/issues/141
    canonical: Option<String>,
    #[serde(default, skip_serializing)]
    pub i18n: HashMap<String, Article>,
}

/// The translation info of an article.
#[derive(Serialize)]
struct Translations<'a> {
    // The locale name.
    name: &'static str,
    // Article slug.
    slug: &'a String,
    // Article path.
    path: &'a Option<String>,
}

impl MetaArticle {
    pub(super) fn has_empty_cover(&self) -> bool {
        self.cover.is_none() || matches!(self.cover.as_ref(), Some(cover) if cover.is_empty())
    }

    fn default_pub_date() -> Date {
        Date::MIN
    }

    fn is_default_pub_date(&self) -> bool {
        self.pub_date == Date::MIN
    }
}

impl std::fmt::Debug for Article {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Article")
            .field("meta", &self.meta)
            .field("i18n", &self.i18n)
            .field("publish", &self.publish)
            .finish()
    }
}

impl Article {
    /// Check whether `author` name is the author of this article.
    pub fn is_author(&self, author: &str) -> bool {
        self.meta
            .author
            .as_ref()
            .map(|inner| inner.is_author(author))
            .unwrap_or_default()
    }

    /// Check whether the article need publish.
    ///
    /// The article need publish in any of two conditions:
    /// - the publish property is true
    /// - in `zine serve` mode
    pub fn need_publish(&self) -> bool {
        self.publish || matches!(current_mode(), Mode::Serve)
    }

    fn get_translations(&self) -> Vec<Translations<'_>> {
        let mut translations = self
            .i18n
            .iter()
            .map(|(locale, article)| Translations {
                name: i18n::get_locale_name(locale)
                    .unwrap_or_else(|| panic!("Currently, we don't support locale: `{locale}`")),
                slug: &article.meta.slug,
                path: &article.meta.path,
            })
            .collect::<Vec<_>>();

        if !translations.is_empty() {
            let zine_data = data::read();
            let site = zine_data.get_site();
            // Add default locale.
            translations.push(Translations {
                name: i18n::get_locale_name(&site.locale).unwrap_or_else(|| {
                    panic!("Currently, we don't support locale: `{}`", site.locale)
                }),
                slug: &self.meta.slug,
                path: &self.meta.path,
            });
            translations.sort_by_key(|t| t.name);
        }
        translations
    }

    fn parse(&mut self, source: &Path) -> Result<()> {
        let file_path = source.join(&self.meta.file);
        self.markdown = fs::read_to_string(&file_path).with_context(|| {
            format!("Failed to read markdown file of `{}`", file_path.display())
        })?;

        // Fallback to file name if no slug specified.
        if self.meta.path.is_none() && self.meta.slug.is_empty() {
            self.meta.slug = self.meta.file.replace(".md", "")
        }
        // Fallback to the default placeholder image if the cover is missing.
        if self.meta.has_empty_cover() {
            let data = data::read();
            self.meta.cover = data.get_theme().default_cover.clone();
        }
        // Ensure the path starts with / if exists.
        if matches!(self.meta.path.as_ref(), Some(path) if !path.starts_with('/')) {
            self.meta.path = Some(format!("/{}", self.meta.path.take().unwrap_or_default()));
        }
        Ok(())
    }

    fn render(&self, env: &Environment, mut context: Context, dest: &Path) -> Result<()> {
        context.insert(
            "meta",
            &Meta {
                title: Cow::Borrowed(&self.meta.title),
                description: Cow::Owned(markdown::extract_description(&self.markdown)),
                url: Some(
                    if let Some(path) = self
                        .meta
                        .path
                        .as_ref()
                        // Remove the prefix slash
                        .and_then(|path| path.strip_prefix('/'))
                    {
                        Cow::Borrowed(path)
                    } else {
                        let issue_slug = context
                            .get("issue")
                            .and_then(|issue| issue.get("slug"))
                            .and_then(|v| v.as_str())
                            .unwrap_or_default();
                        Cow::Owned(format!("{}/{}", issue_slug, self.meta.slug))
                    },
                ),
                image: self.meta.cover.as_deref().map(Cow::Borrowed),
            },
        );
        context.insert("page_type", "article");
        context.insert("article", &self);
        context.insert("canonical_url", &self.canonical);

        let (html, toc) = markdown::render_html_with_toc(&self.markdown);
        context.insert("html", &html);
        context.insert("toc", &toc);

        if let Some(path) = self.meta.path.as_ref() {
            let mut dest = dest.to_path_buf();
            dest.pop();
            engine::render(
                env,
                "article.jinja",
                context,
                dest.join(path.trim_start_matches('/')),
            )
        } else {
            engine::render(env, "article.jinja", context, dest.join(&self.meta.slug))
        }
    }
}

impl Entity for Article {
    fn parse(&mut self, source: &Path) -> Result<()> {
        Article::parse(self, source)?;
        ensure!(
            !self.meta.is_default_pub_date(),
            "`pub_date` is required for article `{}`",
            self.meta.title
        );
        {
            let zine_data = data::read();
            self.topics.iter().for_each(|topic| {
                if !zine_data.is_valid_topic(topic) {
                    println!(
                        "Warning: the topic `{}` is invalid, please declare it in the root `zine.toml`",
                        topic
                    )
                }
            });
        }

        for article in self.i18n.values_mut() {
            // Extend topics from the origin article
            article.topics = self.topics.clone();
            if article.meta.author.is_none() {
                article.meta.author = self.meta.author.clone();
            }
            if article.meta.has_empty_cover() {
                article.meta.cover = self.meta.cover.clone();
            }
            // Fallback to original article date if the `pub_date` is missing
            if article.meta.is_default_pub_date() {
                article.meta.pub_date = self.meta.pub_date;
            }
            Article::parse(article, source)?;
        }
        Ok(())
    }

    fn render(&self, env: &Environment, mut context: Context, dest: &Path) -> Result<()> {
        context.insert("i18n", &self.get_translations());
        Article::render(self, env, context.clone(), dest)?;

        self.i18n.values().par_bridge().for_each(|article| {
            Article::render(article, env, context.clone(), dest).expect("Failed to render article");
        });

        Ok(())
    }
}
