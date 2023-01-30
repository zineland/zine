use std::io::prelude::*;
use std::{
    borrow::Cow,
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{ensure, Context as _, Result};
use serde::{Deserialize, Serialize};
use tera::Context;
use time::Date;

use crate::{
    current_mode, data, engine,
    html::Meta,
    i18n,
    markdown::{self, MarkdownRender},
    Mode,
};

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
    #[serde(with = "crate::helpers::serde_date")]
    #[serde(default = "MetaArticle::default_pub_date")]
    pub pub_date: Date,
}

impl MetaArticle {
    /// Create a new MetaAtricle using Defaults::defaults()
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    /// Set the Title for the article and also set the file based on the Title.
    fn set_title(&mut self, title: &str) -> &mut Self {
        self.title = title.into();
        self.file = self.title.clone().to_lowercase().replace(' ', "-");
        self
    }
    /// Set the Author Ids by parsing a provided string. Names should be simply listed with spaces
    fn set_authors(&mut self, authors: &str) -> Result<&mut Self> {
        if let Ok(authors) = authors.to_string().parse::<AuthorId>() {
            self.author = Some(authors);
            return Ok(self);
        };
        Err(anyhow::anyhow!(
            "Unable to parse string containing author names."
        ))
    }
    fn finalize(&self) -> Self {
        self.to_owned()
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

impl Default for MetaArticle {
    fn default() -> Self {
        Self {
            file: "Give-this-file-a-name".into(),
            // Need more information on what this should be
            slug: "1".into(),
            title: "Give me a Title.".into(),
            path: None,
            author: None,
            cover: None,
            pub_date: Date::MIN,
        }
    }
}

#[cfg(test)]
mod meta_article_tests {

    use crate::entity::{article::MetaArticle, author::AuthorId};
    use time::Date;

    #[test]
    fn test_meta_article_default() {
        let meta_defaults = MetaArticle::default();

        assert_eq!(meta_defaults.file, "Give-this-file-a-name");
        assert_eq!(meta_defaults.slug, "1");
        assert_eq!(meta_defaults.path, None);
        assert_eq!(meta_defaults.title, "Give me a Title.");
        assert_eq!(meta_defaults.cover, None);
        assert_eq!(meta_defaults.pub_date, Date::MIN);
    }
    #[test]
    fn test_meta_article_new() {
        let mut m_a = MetaArticle::new();
        assert_eq!(m_a.file, "Give-this-file-a-name");
        m_a.set_title("This is a test");
        assert_eq!(m_a.title, "This is a test");
        assert_eq!(m_a.file, "this-is-a-test");
        m_a.set_authors("Bob Bas-Man").unwrap();
        //let a = AuthorId::List(vec![String::from("Alice"), String::from("Bob")]);
        assert!(matches!(m_a.author.unwrap(),
                AuthorId::List(names) if names == vec![String::from("Bob"), String::from("Bas-Man")],));
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
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
    pub publish: bool,
    #[serde(default, skip_serializing)]
    pub i18n: HashMap<String, Article>,
}

impl Article {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    /// If you call set_meta. You do not need to call set_title as MetaArticle
    /// contains all the related details.
    fn set_meta(&mut self, article_meta: MetaArticle) -> &mut Self {
        self.meta = article_meta;
        self
    }
    fn set_title(&mut self, title: &str) -> &mut Self {
        self.meta.set_title(title);
        self
    }
    fn set_featured_to_true(&mut self) -> &mut Self {
        self.featured = true;
        self
    }
    fn set_published_to_true(&mut self) -> &mut Self {
        self.publish = true;
        self
    }
    fn append_article_to_toml(&self, path: PathBuf) -> Result<()> {
        // Article zine.toml file must exist
        if !path.exists() {
            Err(anyhow::anyhow!("Issue toml file does not already exists"))?
        };

        let mut file = std::fs::OpenOptions::new().append(true).open(&path)?;

        let toml_str = toml::to_string(&self)?;

        // Code fix as the section does not appear to be added by default.
        file.write_all("[[article]]\n".as_bytes())?;
        file.write_all(toml_str.as_bytes())?;

        Ok(())
    }
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
        if self.meta.cover.is_none() || matches!(&self.meta.cover, Some(cover) if cover.is_empty())
        {
            let data = data::read();
            self.meta.cover = data.get_theme().default_cover.clone();
        }
        // Ensure the path starts with / if exists.
        if matches!(self.meta.path.as_ref(), Some(path) if !path.starts_with('/')) {
            self.meta.path = Some(format!("/{}", self.meta.path.take().unwrap_or_default()));
        }
        Ok(())
    }

    fn render(&self, mut context: Context, dest: &Path) -> Result<()> {
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

        let zine_data = data::read();
        let markdown_config = zine_data.get_markdown_config();
        let mut markdown_render = MarkdownRender::new(markdown_config);
        let html = markdown_render.render_html(&self.markdown);
        markdown_render.rebuild_toc_depth();
        context.insert("html", &html);
        context.insert("toc", &markdown_render.toc);
        drop(zine_data);

        if let Some(path) = self.meta.path.as_ref() {
            let mut dest = dest.to_path_buf();
            dest.pop();
            engine::render(
                "article.jinja",
                &context,
                dest.join(path.trim_start_matches('/')),
            )
        } else {
            engine::render("article.jinja", &context, dest.join(&self.meta.slug))
        }
    }
}

#[cfg(test)]
mod tests_artile_impl {

    use crate::entity::article::{Article, MetaArticle};
    use std::env;

    #[test]
    fn test_default() {
        let article = Article::new();

        assert_eq!(article.featured, false);
        assert_eq!(article.publish, false);
        assert_eq!(article.meta.title, "Give me a Title.");
    }

    #[test]
    fn test_pass_meta() {
        let meta = MetaArticle::new()
            .set_title("This is a great Article")
            .finalize();
        let mut article = Article::new();

        article.set_meta(meta);

        assert_eq!(article.meta.title, "This is a great Article");
    }

    #[test]
    fn test_append_to_file() {
        let meta = MetaArticle::new()
            .set_title("This is a great Article")
            .finalize();
        let mut article = Article::new();

        article.set_meta(meta);
        let work_space = std::path::Path::new("/tmp");
        let path = work_space.to_path_buf().join("test.toml");
        assert!(env::set_current_dir(&work_space).is_ok());
        assert!(std::fs::write(&path, "\n").is_ok());
        assert!(article.append_article_to_toml(path).is_ok())
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
            if article.meta.cover.is_none() {
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

    fn render(&self, mut context: Context, dest: &Path) -> Result<()> {
        context.insert("i18n", &self.get_translations());
        Article::render(self, context.clone(), dest)?;
        for article in self.i18n.values() {
            Article::render(article, context.clone(), dest)?;
        }

        Ok(())
    }
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
