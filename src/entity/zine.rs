use crate::{data, engine, error::ZineError, feed::FeedEntry};
use anyhow::{Context as _, Result};
use genkit::{
    entity::MarkdownConfig,
    helpers::{self, capitalize},
    Context, Entity,
};
use minijinja::{context, Environment};
use rayon::{
    iter::{IntoParallelRefIterator, ParallelBridge, ParallelExtend, ParallelIterator},
    prelude::IntoParallelRefMutIterator,
    slice::ParallelSliceMut,
};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fs,
    path::{Component, Path},
};
use walkdir::WalkDir;

use super::{Author, Issue, List, MetaArticle, Page, Site, Theme, Topic};

/// The root zine entity config.
///
/// It parsed from the root directory's `zine.toml`.
#[derive(Deserialize)]
pub struct Zine {
    pub site: Site,
    #[serde(default)]
    pub theme: Theme,
    #[serde(default)]
    pub authors: BTreeMap<String, Author>,
    #[serde(default)]
    #[serde(rename = "issue")]
    pub issues: Vec<Issue>,
    #[serde(default)]
    pub topics: BTreeMap<String, Topic>,
    #[serde(skip)]
    pub pages: Vec<Page>,
    #[serde(default)]
    #[serde(rename = "markdown")]
    pub markdown_config: MarkdownConfig,
}

impl std::fmt::Debug for Zine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Zine")
            .field("site", &self.site)
            .field("theme", &self.theme)
            .field("issues", &self.issues)
            .finish()
    }
}

// A [`MetaArticle`] and issue info pair.
// Naming is hard, give it a better name?
#[derive(Serialize)]
struct ArticleRef<'a> {
    article: &'a MetaArticle,
    issue_title: &'a String,
    issue_slug: &'a String,
}

impl Zine {
    /// Parse Zine instance from the root zine.toml file.
    pub fn parse_from_toml<P: AsRef<Path>>(source: P) -> Result<Zine> {
        let source = source.as_ref().join(crate::ZINE_FILE);
        let content = fs::read_to_string(&source)
            .with_context(|| format!("Failed to read `{}`", source.display()))?;

        Ok(toml::from_str::<Zine>(&content).map_err(|err| {
            let value = toml::from_str::<toml::Value>(&content)
                .unwrap_or_else(|_| panic!("Parse `{}` failed", source.display()));
            if value.get("site").is_some() {
                ZineError::InvalidRootTomlFile(err)
            } else {
                ZineError::NotRootTomlFile
            }
        })?)
    }

    /// Parsing issue entities from dir.
    pub fn parse_issue_from_dir(&mut self, source: &Path) -> Result<()> {
        let content_dir = source.join(crate::ZINE_CONTENT_DIR);
        if !content_dir.exists() {
            println!(
                "`{}` fold not found, creating it...",
                crate::ZINE_CONTENT_DIR
            );
            fs::create_dir_all(&content_dir)?;
        }

        for entry in WalkDir::new(&content_dir).contents_first(true).into_iter() {
            let entry = entry?;
            if entry.file_name() != crate::ZINE_FILE {
                continue;
            }
            let content = fs::read_to_string(entry.path()).with_context(|| {
                format!(
                    "Failed to parse `zine.toml` of `{}`",
                    entry.path().display()
                )
            })?;
            let mut issue = toml::from_str::<Issue>(&content)?;
            let dir = entry
                .path()
                .components()
                .fold(Vec::new(), |mut dir, component| {
                    let name = component.as_os_str();
                    if !dir.is_empty() && name != crate::ZINE_FILE {
                        dir.push(name.to_string_lossy().to_string());
                        return dir;
                    }

                    if matches!(component, Component::Normal(c) if c == crate::ZINE_CONTENT_DIR ) {
                        // a empty indicator we should start collect the components
                        dir.push(String::new());
                    }
                    dir
                });
            // skip the first empty indicator
            issue.dir = dir[1..].join("/");
            self.issues.push(issue);
        }

        Ok(())
    }

    pub fn get_issue_by_number(&self, number: u32) -> Option<&Issue> {
        self.issues.iter().find(|issue| issue.number == number)
    }

    // Get the article metadata list by author id, sorted by descending order of publishing date.
    fn get_articles_by_author(&self, author_id: &str) -> Vec<ArticleRef> {
        let mut items = self
            .issues
            .par_iter()
            .flat_map(|issue| {
                issue
                    .articles()
                    .into_iter()
                    .flat_map(|article| {
                        let mut articles = vec![article];
                        // including translation articles
                        articles.extend(article.i18n.values());
                        articles
                    })
                    .filter_map(|article| {
                        if article.is_author(author_id) {
                            Some(ArticleRef {
                                article: &article.meta,
                                issue_title: &issue.title,
                                issue_slug: &issue.slug,
                            })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        items.par_sort_unstable_by(|a, b| b.article.pub_date.cmp(&a.article.pub_date));
        items
    }

    // Get the article meta list by topic id
    fn get_articles_by_topic(&self, topic: &str) -> Vec<ArticleRef> {
        let mut items = self
            .issues
            .par_iter()
            .flat_map(|issue| {
                issue
                    .articles()
                    .iter()
                    .filter_map(|article| {
                        if article.topics.iter().any(|t| t == topic) {
                            Some(ArticleRef {
                                article: &article.meta,
                                issue_title: &issue.title,
                                issue_slug: &issue.slug,
                            })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        items.par_sort_unstable_by(|a, b| b.article.pub_date.cmp(&a.article.pub_date));
        items
    }

    // Get author list.
    fn authors(&self) -> Vec<Author> {
        self.authors.values().cloned().collect()
    }

    fn all_articles(&self) -> Vec<(String, MetaArticle)> {
        self.issues
            .par_iter()
            .flat_map(|issue| {
                issue
                    .articles()
                    .iter()
                    .map(|article| (issue.slug.clone(), article.meta.clone()))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// Get latest `limit` number of articles in all issues.
    /// Sort by date in descending order.
    pub fn latest_feed_entries(&self, limit: usize) -> Vec<FeedEntry> {
        let mut entries = self
            .issues
            .par_iter()
            .flat_map(|issue| {
                let mut entries = issue
                    .articles()
                    .iter()
                    .map(|article| FeedEntry {
                        title: &article.meta.title,
                        url: if let Some(path) = article.meta.path.as_ref() {
                            format!("{}{}", self.site.url, path)
                        } else {
                            format!("{}/{}/{}", self.site.url, issue.slug, article.meta.slug)
                        },
                        content: &article.markdown,
                        author: &article.meta.author,
                        date: Some(article.meta.pub_date),
                    })
                    .collect::<Vec<_>>();

                // Add issue intro article into feed
                if issue.need_publish() {
                    if let Some(content) = issue.intro.as_ref() {
                        entries.push(FeedEntry {
                            title: &issue.title,
                            url: format!("{}/{}", self.site.url, issue.slug),
                            content,
                            author: &None,
                            date: issue.pub_date,
                        })
                    }
                }
                entries
            })
            .collect::<Vec<_>>();

        // Sort by date in descending order.
        entries.par_sort_unstable_by(|a, b| match (a.date, b.date) {
            (Some(a_date), Some(b_date)) => b_date.cmp(&a_date),
            _ => Ordering::Equal,
        });
        entries.into_iter().take(limit).collect()
    }

    /// Get `sitemap.xml` entries.
    pub fn sitemap_entries(&self) -> Vec<String> {
        let base_url = &self.site.url;
        // Sitemap URL must begin with the protocol (such as http)
        // and end with a trailing slash.
        // https://www.sitemaps.org/protocol.html
        let mut entries = vec![format!("{}/", base_url)];

        // Issues and articles
        for issue in &self.issues {
            entries.push(format!("{}/{}/", base_url, issue.slug));
            let articles = issue
                .articles()
                .into_iter()
                .par_bridge()
                .flat_map(|article| {
                    let mut articles = vec![article];
                    // including translation articles
                    articles.extend(article.i18n.values());
                    articles
                })
                .map(|article| {
                    if let Some(path) = article.meta.path.as_ref() {
                        format!("{}{}", base_url, path)
                    } else {
                        format!("{}/{}/{}", base_url, issue.slug, article.meta.slug)
                    }
                });
            entries.par_extend(articles);
        }

        // Authors
        entries.push(format!("{}/authors/", base_url));
        entries.par_extend(
            self.authors
                .par_iter()
                .map(|(id, _)| format!("{}/@{}/", base_url, id.to_lowercase())),
        );

        // Topics
        if !self.topics.is_empty() {
            entries.push(format!("{}/topics/", base_url));
            entries.par_extend(
                self.topics
                    .par_iter()
                    .map(|(id, _)| format!("{}/topic/{}/", base_url, id.to_lowercase())),
            );
        }

        // Pages
        entries.par_extend(
            self.pages
                .par_iter()
                .map(|page| format!("{}/{}/", base_url, page.slug())),
        );
        entries
    }
}

impl Entity for Zine {
    fn parse(&mut self, source: &Path) -> Result<()> {
        self.theme.parse(source)?;

        self.topics.par_iter_mut().try_for_each(|(id, topic)| {
            topic.id = id.clone();
            topic.parse(source)
        })?;

        {
            let mut zine_data = data::write();
            zine_data
                .set_theme(self.theme.clone())
                .set_site(self.site.clone())
                .set_topics(self.topics.keys().cloned().collect());
        }

        self.parse_issue_from_dir(source)?;

        self.issues.parse(source)?;
        // Sort all issues by number.
        self.issues.par_sort_unstable_by_key(|s| s.number);

        if self.authors.is_empty() {
            println!("Warning: no author specified in [authors] of root `zine.toml`.");
        } else {
            self.authors.par_iter_mut().try_for_each(|(id, author)| {
                author.id = id.clone();
                // Fallback to default zine avatar if neccessary.
                if author.avatar.is_none()
                    || matches!(&author.avatar, Some(avatar) if avatar.is_empty())
                {
                    author.avatar = self.theme.default_avatar.clone();
                }

                // Fallback to capitalized id if missing.
                if author.name.is_none() {
                    author.name = Some(capitalize(&author.id));
                }
                author.parse(source)
            })?;
        }

        // Parse pages
        let page_dir = source.join("pages");
        if page_dir.exists() {
            // Parallelize pages dir walk
            self.pages = WalkDir::new(&page_dir)
                .into_iter()
                .par_bridge()
                .try_fold_with(vec![], |mut pages, entry| {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() {
                        let markdown = fs::read_to_string(path).with_context(|| {
                            format!("Failed to read markdown file of `{}`", path.display())
                        })?;
                        pages.push(Page {
                            markdown,
                            file_path: path.strip_prefix(&page_dir)?.to_owned(),
                        });
                    }
                    anyhow::Ok(pages)
                })
                .try_reduce_with(|mut pages, chuncks| {
                    pages.par_extend(chuncks);
                    anyhow::Ok(pages)
                })
                .transpose()?
                .unwrap_or_default();
        }
        Ok(())
    }

    fn render(&self, env: &Environment, mut context: Context, dest: &Path) -> Result<()> {
        context.insert("site", &self.site);

        // Render all authors pages.
        let authors = self.authors();
        let mut author_list = List::author_list();
        authors.iter().try_for_each(|author| {
            let articles = self.get_articles_by_author(&author.id);
            author_list.push_author(author, articles.len());

            let mut context = context.clone();
            context.insert("articles", &articles);
            author
                .render(env, context, dest)
                .expect("Failed to render author page");

            anyhow::Ok(())
        })?;
        // Render author list page.
        author_list
            .render(env, context.clone(), dest)
            .expect("Failed to render author list page");

        {
            let mut zine_data = data::write();
            zine_data
                .set_authors(authors)
                .set_articles(self.all_articles());
        }

        // Render all issues pages.
        self.issues
            .render(env, context.clone(), dest)
            .expect("Failed to render issues");

        // Render all topic pages
        let topic_dest = dest.join("topic");
        let mut topic_list = List::topic_list();
        self.topics
            .values()
            .try_for_each(|topic| {
                let mut context = context.clone();
                let articles = self.get_articles_by_topic(&topic.id);
                topic_list.push_topic(topic, articles.len());
                context.insert("articles", &articles);
                topic.render(env, context, &topic_dest)
            })
            .expect("Failed to render topic pages");
        // Render topic list page
        topic_list
            .render(env, context.clone(), dest)
            .expect("Failed to render topic list page");

        // Render other pages.
        self.pages
            .render(env, context.clone(), dest)
            .expect("Failed to render pages");

        // Render home page.
        let issues = self
            .issues
            .par_iter()
            .filter(|issue| issue.need_publish())
            .map(|issue| {
                context! {
                    slug => issue.slug,
                    title => issue.title,
                    number => issue.number,
                    pub_date => issue.pub_date.as_ref().map(helpers::format_date),
                    articles => issue.featured_articles(),
                }
            })
            .collect::<Vec<_>>();
        context.insert("issues", &issues);
        engine::render(env, "index.jinja", context, dest).expect("Failed to render home page");
        Ok(())
    }
}
