use anyhow::Result;
use rayon::{
    iter::{IntoParallelRefIterator, ParallelBridge, ParallelExtend, ParallelIterator},
    slice::ParallelSliceMut,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::Path,
};
use tera::Context;
use walkdir::WalkDir;

use crate::{data, feed::FeedEntry, Entity, Render};

use super::{Author, AuthorList, Issue, MetaArticle, Page, Site, Theme};

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
    #[serde(rename = "page")]
    #[serde(default)]
    pub pages: Vec<Page>,
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

#[derive(Serialize)]
struct AuthorArticle<'a> {
    article: &'a MetaArticle,
    issue_title: &'a String,
    issue_slug: &'a String,
}

impl Zine {
    // Query the article metadata list by author id, sorted by descending order of publishing date.
    fn query_articles_by_author(&self, author_id: &str) -> Vec<AuthorArticle> {
        let mut items = self
            .issues
            .par_iter()
            .flat_map(|issue| {
                issue
                    .articles
                    .iter()
                    .filter_map(|article| {
                        if article.is_author(author_id) {
                            Some(AuthorArticle {
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
        self.authors
            .iter()
            .map(|(id, author)| Author {
                id: id.to_owned(),
                ..author.to_owned()
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
                issue
                    .articles
                    .iter()
                    .map(|article| FeedEntry {
                        title: &article.meta.title,
                        url: format!("{}/{}/{}", self.site.url, issue.slug, article.slug()),
                        content: &article.markdown,
                        author: &article.meta.author,
                        date: &article.meta.pub_date,
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        // Sort by date in descending order.
        entries.par_sort_unstable_by(|a, b| b.date.cmp(a.date));
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
            entries.par_extend(
                issue
                    .articles
                    .par_iter()
                    .map(|article| format!("{}/{}/{}/", base_url, issue.slug, article.slug())),
            )
        }

        // Authors
        entries.push(format!("{}/authors/", base_url));
        entries.par_extend(
            self.authors
                .par_iter()
                .map(|(id, _)| format!("{}/@{}/", base_url, id.to_lowercase())),
        );

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
        if self.authors.is_empty() {
            println!("Warn: no author specified in [authors] of root `zine.toml`.");
        } else {
            self.authors
                .values_mut()
                .try_for_each(|author| author.parse(source))?;
        }

        self.theme.parse(source)?;
        self.issues.parse(source)?;
        // Sort all issues by number.
        self.issues.par_sort_unstable_by_key(|s| s.number);

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
                        let markdown = fs::read_to_string(path)?;
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

    fn render(&self, mut context: Context, dest: &Path) -> Result<()> {
        context.insert("theme", &self.theme);
        context.insert("site", &self.site);

        // Render all authors pages.
        let authors = self.authors();
        let mut author_list = AuthorList::default();
        for author in &authors {
            let articles = self.query_articles_by_author(&author.id);
            author_list.record_author(author, articles.len());

            let mut context = context.clone();
            context.insert("articles", &articles);
            author.render(context, dest)?;
        }

        // Render author list page.
        author_list.render(context.clone(), dest)?;
        data::get().set_authors(authors);

        // Render all issues pages.
        self.issues.render(context.clone(), dest)?;

        // Render other pages.
        self.pages.render(context.clone(), dest)?;

        // Render home page.
        context.insert("issues", &self.issues);
        // `article_map` is the issue number and issue's featured articles map.
        let article_map = self
            .issues
            .iter()
            .map(|issue| (issue.number, issue.featured_articles()))
            .collect::<HashMap<u32, Vec<_>>>();
        context.insert("article_map", &article_map);
        Render::render("index.jinja", &context, dest)?;
        Ok(())
    }
}
