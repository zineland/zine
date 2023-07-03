use once_cell::sync::OnceCell;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::entity::{Author, MetaArticle, Site, Theme};

static ZINE_DATA: OnceCell<RwLock<ZineData>> = OnceCell::new();

pub fn load() {
    ZINE_DATA.get_or_init(|| RwLock::new(ZineData::default()));
}

pub fn read() -> RwLockReadGuard<'static, ZineData> {
    ZINE_DATA.get().unwrap().read()
}

pub fn write() -> RwLockWriteGuard<'static, ZineData> {
    ZINE_DATA.get().unwrap().write()
}

#[derive(Debug, Default)]
pub struct ZineData {
    authors: Vec<Author>,
    // Issue slug and article pair list.
    articles: Vec<(String, MetaArticle)>,
    // The topic name list.
    topics: Vec<String>,
    site: Site,
    theme: Theme,
}

impl ZineData {
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

    pub fn get_theme(&self) -> &Theme {
        &self.theme
    }

    pub fn is_valid_topic(&self, topic: &str) -> bool {
        self.topics.iter().any(|t| t.eq_ignore_ascii_case(topic))
    }
}
