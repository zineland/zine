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

use super::{Author, MetaArticle, Page, Season, Site, Theme};

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
    #[serde(rename = "season")]
    pub seasons: Vec<Season>,
    #[serde(rename = "page")]
    #[serde(default)]
    pub pages: Vec<Page>,
}

impl std::fmt::Debug for Zine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Zine")
            .field("site", &self.site)
            .field("theme", &self.theme)
            .field("seasons", &self.seasons)
            .finish()
    }
}

#[derive(Serialize)]
struct AuthorArticle<'a> {
    article: &'a MetaArticle,
    season_title: &'a String,
    season_slug: &'a String,
}

impl Zine {
    // Query the article metadata list by author id, sorted by descending order of publishing date.
    fn query_articles_by_author(&self, author_id: &str) -> Vec<AuthorArticle> {
        let mut items = self
            .seasons
            .par_iter()
            .flat_map(|season| {
                season
                    .articles
                    .iter()
                    .filter_map(|article| {
                        if article.is_author(author_id) {
                            Some(AuthorArticle {
                                article: &article.meta,
                                season_title: &season.title,
                                season_slug: &season.slug,
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

    /// Get latest `limit` number of articles in all seasons.
    /// Sort by date in descending order.
    pub fn latest_feed_entries(&self, limit: usize) -> Vec<FeedEntry> {
        let mut entries = self
            .seasons
            .par_iter()
            .flat_map(|season| {
                season
                    .articles
                    .iter()
                    .map(|article| FeedEntry {
                        title: &article.meta.title,
                        url: format!("{}/{}/{}", self.site.url, season.slug, article.slug()),
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
        // Sitemap URL must begin with the protocol (such as http)
        // and end with a trailing slash.
        // https://www.sitemaps.org/protocol.html
        let mut entries = vec![format!("{}/", &self.site.url)];
        for season in &self.seasons {
            entries.push(format!("{}/{}/", self.site.url, season.slug));
            entries.par_extend(
                season.articles.par_iter().map(|article| {
                    format!("{}/{}/{}/", self.site.url, season.slug, article.slug())
                }),
            )
        }

        entries.par_extend(
            self.pages
                .par_iter()
                .map(|page| format!("{}/{}/", self.site.url, page.slug())),
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
        self.seasons.parse(source)?;
        // Sort all seasons by number.
        self.seasons.par_sort_unstable_by_key(|s| s.number);

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
        for author in &authors {
            let mut context = context.clone();
            context.insert("articles", &self.query_articles_by_author(&author.id));
            author.render(context, dest)?;
        }
        data::get().set_authors(authors);

        // Render all seasons pages.
        self.seasons.render(context.clone(), dest)?;

        // Render other pages.
        self.pages.render(context.clone(), dest)?;

        // Render home page.
        context.insert("seasons", &self.seasons);
        // `article_map` is the season number and season's featured articles map.
        let article_map = self
            .seasons
            .iter()
            .map(|season| (season.number, season.featured_articles()))
            .collect::<HashMap<u32, Vec<_>>>();
        context.insert("article_map", &article_map);
        Render::render("index.jinja", &context, dest)?;
        Ok(())
    }
}
