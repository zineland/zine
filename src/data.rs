use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
    path::Path,
    sync::Arc,
};

use anyhow::Result;
use dashmap::DashMap;
use once_cell::sync::OnceCell;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use serde::{
    de,
    ser::{SerializeMap, SerializeSeq},
    Deserialize, Serialize,
};
use tokio::sync::watch::{self, Receiver};

use crate::{
    entity::{Author, MarkdownConfig, MetaArticle, Site, Theme},
    helpers, html,
};

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

#[derive(Clone, Debug)]
pub struct UrlPreviewInfo {
    pub title: String,
    pub description: String,
    pub image: Option<String>,
}

impl Serialize for UrlPreviewInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(3))?;
        seq.serialize_element(&self.title)?;
        seq.serialize_element(&self.description)?;
        if let Some(image) = self.image.as_ref() {
            seq.serialize_element(image)?;
        } else {
            seq.serialize_element("")?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for UrlPreviewInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(UrlPreviewInfoVisitor)
    }
}

struct UrlPreviewInfoVisitor;

impl<'de> de::Visitor<'de> for UrlPreviewInfoVisitor {
    type Value = UrlPreviewInfo;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("2 or 3 elements tuple")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let (title, description, image) = (
            seq.next_element()?.unwrap_or_default(),
            seq.next_element()?.unwrap_or_default(),
            seq.next_element()?,
        );
        Ok(UrlPreviewInfo {
            title,
            description,
            image,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZineData {
    #[serde(skip)]
    authors: Vec<Author>,
    // Issue slug and article pair list.
    #[serde(skip)]
    articles: Vec<(String, MetaArticle)>,
    // The topic name list.
    #[serde(skip)]
    topics: Vec<String>,
    #[serde(skip)]
    site: Site,
    #[serde(skip)]
    markdown_config: MarkdownConfig,
    #[serde(skip)]
    theme: Theme,
    // The preview tasks.
    #[serde(skip)]
    preview_tasks: DashMap<String, Receiver<Option<PreviewEvent>>>,
    // All url preview data.
    url_previews: Arc<DashMap<String, UrlPreviewInfo>>,
}

// Implement Serialize manually to keep urlPreviews ordered.
impl Serialize for ZineData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut url_previews = BTreeMap::new();
        self.url_previews.iter().for_each(|kv| {
            let (key, value) = kv.pair();
            url_previews.insert(key.to_owned(), value.to_owned());
        });

        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("urlPreviews", &url_previews)?;
        map.end()
    }
}

#[derive(Debug, Clone)]
pub enum PreviewEvent {
    Finished(UrlPreviewInfo),
    Failed(String),
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
                topics: Vec::default(),
                site: Site::default(),
                markdown_config: MarkdownConfig::default(),
                theme: Theme::default(),
                url_previews: Arc::new(DashMap::default()),
                preview_tasks: DashMap::default(),
            })
        }
    }

    pub fn get_all_previews(&self) -> Arc<DashMap<String, UrlPreviewInfo>> {
        Arc::clone(&self.url_previews)
    }

    pub fn get_preview(&self, url: &str) -> Option<UrlPreviewInfo> {
        self.url_previews.get(url).map(|u| u.to_owned())
    }

    /// Preview url asynchronously, return a tuple.
    /// The first bool argument indicating whether is a first time previewing.
    /// The second argument is the receiver to wait preview event finished.
    pub fn preview_url(&self, url: &str) -> (bool, Receiver<Option<PreviewEvent>>) {
        if let Some(rx) = self.preview_tasks.get(url) {
            // In the preview queue.
            (false, rx.clone())
        } else {
            let (tx, rx) = watch::channel::<Option<PreviewEvent>>(None);
            // Not in the preview queue, enqueue the preview task.
            self.preview_tasks.insert(url.to_owned(), rx.clone());

            let url = url.to_owned();
            let list = Arc::clone(&self.url_previews);
            // Spawn a background task to preview the url.
            tokio::spawn(async move {
                match helpers::fetch_url(&url).await {
                    Ok(html) => {
                        let meta = html::parse_html_meta(html);
                        let info = UrlPreviewInfo {
                            title: meta.title.into_owned(),
                            description: meta.description.into_owned(),
                            image: meta.image.as_ref().map(|image| image.to_string()),
                        };

                        list.insert(url, info.clone());
                        tx.send(Some(PreviewEvent::Finished(info)))
                    }
                    Err(err) => tx.send(Some(PreviewEvent::Failed(err.to_string()))),
                }
            });
            (true, rx)
        }
    }

    pub fn set_authors(&mut self, authors: Vec<Author>) -> &mut Self {
        self.authors = authors;
        self
    }

    pub fn set_topics(&mut self, topics: Vec<String>) -> &mut Self {
        self.topics = topics;
        self
    }

    pub fn set_articles(&mut self, articles: Vec<(String, MetaArticle)>) -> &mut Self {
        self.articles = articles;
        self
    }

    pub fn set_site(&mut self, site: Site) -> &mut Self {
        self.site = site;
        self
    }

    pub fn set_markdown_config(&mut self, config: MarkdownConfig) -> &mut Self {
        self.markdown_config = config;
        self
    }

    pub fn set_theme(&mut self, theme: Theme) -> &mut Self {
        self.theme = theme;
        self
    }

    pub fn get_authors(&self) -> Vec<&Author> {
        self.authors.iter().by_ref().collect()
    }

    pub fn get_author_by_id(&self, author_id: &str) -> Option<&Author> {
        self.authors
            .iter()
            .find(|author| author.id.eq_ignore_ascii_case(author_id))
    }

    pub fn get_article_by_path(&self, article_path: &str) -> Option<MetaArticle> {
        self.articles
            .iter()
            .find_map(|(issue_slug, article)| {
                if article.path.as_deref() == Some(article_path)
                    || format!("/{}/{}", issue_slug, article.slug) == article_path
                {
                    Some(article)
                } else {
                    None
                }
            })
            .cloned()
    }

    pub fn get_site(&self) -> &Site {
        &self.site
    }

    pub fn get_markdown_config(&self) -> &MarkdownConfig {
        &self.markdown_config
    }

    pub fn get_theme(&self) -> &Theme {
        &self.theme
    }

    pub fn is_valid_topic(&self, topic: &str) -> bool {
        self.topics.iter().any(|t| t.eq_ignore_ascii_case(topic))
    }

    fn export_to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}
