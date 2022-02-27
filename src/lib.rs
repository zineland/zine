use serde::{Deserialize, Serialize};

mod build;
mod parser;
mod render;

pub use build::Builder;
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Site {
    pub name: String,
    pub logo: Option<String>,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Article {
    pub file: String,
    // The slug after this artcile rendered.
    // Default to file name if no slug specified.
    pub slug: Option<String>,
    pub title: String,
    pub author: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing)]
    pub html: String,
    // TODO: deserialize to OffsetDateTime
    pub pub_date: String,
    #[serde(default)]
    pub publish: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    pub slug: String,
    pub name: String,
    pub content: String,
}

impl Article {
    pub fn slug(&self) -> String {
        self.slug
            .as_ref()
            .cloned()
            .unwrap_or_else(|| self.file.replace(".md", ""))
    }
}
