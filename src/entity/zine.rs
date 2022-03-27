use anyhow::Result;
use rayon::{
    iter::{IntoParallelRefIterator, ParallelBridge, ParallelExtend, ParallelIterator},
    slice::ParallelSliceMut,
};
use serde::Deserialize;
use std::{fs, path::Path};
use tera::Context;
use walkdir::WalkDir;

use crate::{feed::FeedEntry, Entity, Render};

use super::{Page, Season, Site, Theme};

/// The root zine entity config.
///
/// It parsed from the root directory's `zine.toml`.
#[derive(Deserialize)]
pub struct Zine {
    pub site: Site,
    #[serde(default)]
    pub theme: Theme,
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

impl Zine {
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
                        title: &article.title,
                        url: format!("{}/{}/{}", self.site.url, season.slug, article.slug()),
                        content: &article.markdown,
                        author: &article.author,
                        date: &article.pub_date,
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        // Sort by date in descending order.
        entries.par_sort_unstable_by(|a, b| b.date.cmp(a.date));
        entries.into_iter().take(limit).collect()
    }
}

impl Entity for Zine {
    fn parse(&mut self, source: &Path) -> Result<()> {
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
        // Render all seasons pages.
        self.seasons.render(context.clone(), dest)?;

        // Render other pages.
        self.pages.render(context.clone(), dest)?;

        // Render home page.
        context.insert("seasons", &self.seasons);
        Render::render("index.jinja", &context, dest)?;
        Ok(())
    }
}
