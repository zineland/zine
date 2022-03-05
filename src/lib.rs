use std::path::PathBuf;

use serde::{Deserialize, Serialize};

mod build;
mod parser;
mod render;

pub use build::Builder;
pub use parser::Parser;

#[derive(Debug, Deserialize)]
pub struct Zine {
    pub site: Site,
    pub theme: Theme,
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
    #[serde(rename(deserialize = "menu"))]
    #[serde(default)]
    pub menus: Vec<Menu>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "kebab-case"))]
pub struct Theme {
    // The primary color.
    #[serde(default = "Theme::default_primary_color")]
    pub primary_color: String,
    #[serde(default = "Theme::default_text_color")]
    pub primary_text_color: String,
    #[serde(default = "Theme::default_link_color")]
    pub primary_link_color: String,
    // The background color.
    #[serde(default = "Theme::default_secondary_color")]
    pub secondary_color: String,
    // The background image url.
    #[serde(default)]
    pub background_image: Option<String>,
    // The custom footer template path, will be parsed to html.
    pub footer_template: Option<String>,
}

impl Theme {
    pub const DEFAULT_PRIMARY_COLOR: &'static str = "#2563eb";
    pub const DEFAULT_TEXT_COLOR: &'static str = "#ffffff";
    pub const DEFAULT_LINK_COLOR: &'static str = "#2563eb";
    pub const DEFAULT_SECONDARY_COLOR: &'static str = "#eff3f7";

    fn default_primary_color() -> String {
        Self::DEFAULT_PRIMARY_COLOR.to_string()
    }

    fn default_text_color() -> String {
        Self::DEFAULT_TEXT_COLOR.to_string()
    }

    fn default_link_color() -> String {
        Self::DEFAULT_LINK_COLOR.to_string()
    }

    fn default_secondary_color() -> String {
        Self::DEFAULT_SECONDARY_COLOR.to_string()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Menu {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Season {
    pub slug: String,
    pub number: u32,
    pub title: String,
    pub summary: Option<String>,
    pub cover: Option<String>,
    pub path: String,
    #[serde(rename(deserialize = "article"))]
    #[serde(default)]
    pub articles: Vec<Article>,
}

#[derive(Serialize, Deserialize)]
pub struct Article {
    pub file: String,
    // The slug after this artcile rendered.
    // Default to file name if no slug specified.
    pub slug: Option<String>,
    pub title: String,
    pub author: Option<String>,
    pub cover: Option<String>,
    #[serde(default)]
    pub html: String,
    // TODO: deserialize to OffsetDateTime
    pub pub_date: String,
    // Wheter the article is an featured article.
    // Featured article will display in home page.
    #[serde(default)]
    pub featured: bool,
    #[serde(default)]
    pub publish: bool,
}

impl std::fmt::Debug for Article {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Article")
            .field("file", &self.file)
            .field("slug", &self.slug)
            .field("title", &self.title)
            .field("author", &self.author)
            .field("cover", &self.cover)
            .field("pub_date", &self.pub_date)
            .field("publish", &self.publish)
            .finish()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    // pub file_name: String,
    pub html: String,
    // Relative path of page file.
    pub file_path: PathBuf,
}

impl Article {
    pub fn slug(&self) -> String {
        self.slug
            .as_ref()
            .cloned()
            .unwrap_or_else(|| self.file.replace(".md", ""))
    }
}

impl Page {
    pub fn slug(&self) -> String {
        self.file_path.to_str().unwrap().replace(".md", "")
    }
}
