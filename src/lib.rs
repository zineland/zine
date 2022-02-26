use serde::Deserialize;

mod parser;

pub use parser::Parser;

#[derive(Debug, Deserialize)]
pub struct Zine {
    pub site: Site,
    #[serde(default)]
    #[serde(rename = "season")]
    pub seasons: Vec<Season>,
    #[serde(rename = "page")]
    #[serde(default)]
    pub pages: Vec<Page>,
}

#[derive(Debug, Deserialize)]
pub struct Site {
    pub name: String,
    pub logo: Option<String>,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Season {
    pub slug: String,
    pub number: u32,
    pub title: String,
    pub summary: Option<String>,
    pub path: String,
    #[serde(rename = "article")]
    #[serde(default)]
    pub articles: Vec<Article>,
}

#[derive(Debug, Deserialize)]
pub struct Article {
    pub slug: String,
    pub file: String,
    pub title: String,
    pub author: Option<String>,
    #[serde(default)]
    pub markdown: String,
    // TODO: deserialize to OffsetDateTime
    pub pub_date: String,
    #[serde(default)]
    pub publish: bool,
}

#[derive(Debug, Deserialize)]
pub struct Page {
    pub slug: String,
    pub name: String,
    pub content: String,
}
