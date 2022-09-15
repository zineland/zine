use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
    path::Path,
};

use anyhow::Result;
use once_cell::sync::OnceCell;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use serde::{Deserialize, Serialize};

use crate::entity::{Author, MarkdownConfig, MetaArticle};

static ZINE_DATA: OnceCell<RwLock<ZineData>> = OnceCell::new();

pub fn load<P: AsRef<Path>>(path: P) {
    ZINE_DATA.get_or_init(|| RwLock::new(ZineData::new(path.as_ref()).unwrap()));
}

pub fn read() -> RwLockReadGuard<'static, ZineData> {
    ZINE_DATA.get().unwrap().read()
}

pub fn write() -> RwLockWriteGuard<'static, ZineData> {
    ZINE_DATA.get().unwrap().write()
}

/// Export all data into the `zine-data.json` file.
/// If the data is empty, we never create the `zine-data.json` file.
pub fn export<P: AsRef<Path>>(path: P) -> Result<()> {
    let data = read();
    if !data.url_previews.is_empty() {
        let mut file = File::create(path.as_ref().join("zine-data.json"))?;
        file.write_all(data.export_to_json()?.as_bytes())?;
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZineData {
    #[serde(skip)]
    authors: Vec<Author>,
    // Issue slug and article pair list.
    #[serde(skip)]
    articles: Vec<(String, MetaArticle)>,
    #[serde(skip)]
    markdown_config: MarkdownConfig,
    url_previews: BTreeMap<String, (String, String)>,
}

impl ZineData {
    pub fn new(source: impl AsRef<Path>) -> Result<Self> {
        let path = source.as_ref().join("zine-data.json");
        if path.exists() {
            let json = fs::read_to_string(path)?;
            Ok(serde_json::from_str(&json)?)
        } else {
            Ok(ZineData {
                authors: Vec::default(),
                articles: Vec::default(),
                markdown_config: MarkdownConfig::default(),
                url_previews: BTreeMap::default(),
            })
        }
    }

    pub fn url_previews(&self) -> &BTreeMap<String, (String, String)> {
        &self.url_previews
    }

    pub fn insert_url_preview(&mut self, url: &str, preview: (String, String)) {
        self.url_previews.insert(url.to_owned(), preview);
    }

    pub fn set_authors(&mut self, authors: Vec<Author>) {
        self.authors = authors;
    }

    pub fn set_articles(&mut self, articles: Vec<(String, MetaArticle)>) {
        self.articles = articles;
    }

    pub fn set_markdown_config(&mut self, config: MarkdownConfig) {
        self.markdown_config = config;
    }

    pub fn get_author_by_id(&self, author_id: &str) -> Option<&Author> {
        self.authors
            .iter()
            .find(|author| author.id.eq_ignore_ascii_case(author_id))
    }

    pub fn get_article_by_path<P: AsRef<Path>>(&self, article_path: P) -> Option<MetaArticle> {
        self.articles
            .iter()
            .find_map(|(issue_slug, article)| {
                if Path::new("/").join(issue_slug).join(article.slug()) == article_path.as_ref() {
                    Some(article)
                } else {
                    None
                }
            })
            .cloned()
    }

    pub fn get_markdown_config(&self) -> &MarkdownConfig {
        &self.markdown_config
    }

    fn export_to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}
