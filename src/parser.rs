use anyhow::Result;
use pulldown_cmark::{html, Options, Parser as MarkdownParser};
use serde::Deserialize;
use std::{fs, path::PathBuf};
use walkdir::WalkDir;

use crate::{Article, Page, Zine};

static ZINE_FILE: &str = "zine.toml";

#[derive(Debug)]
pub struct Parser {
    path: PathBuf,
}

// Representing a zine.toml file for season.
#[derive(Debug, Deserialize)]
struct SeasonFile {
    #[serde(rename = "article")]
    articles: Vec<Article>,
}

impl Parser {
    pub fn new(path: &str) -> Self {
        Parser {
            path: PathBuf::from(path),
        }
    }

    pub fn parse(&self) -> Result<Zine> {
        let content = fs::read_to_string(&self.path.join(ZINE_FILE))?;
        let mut site = toml::from_str::<Zine>(&content)?;
        for season in &mut site.seasons {
            season.articles = self.parse_articles(&season.path)?;
        }
        site.pages = self.parse_pages()?;
        Ok(site)
    }

    fn parse_articles(&self, season_path: &str) -> Result<Vec<Article>> {
        let dir = self.path.join(season_path);
        let content = fs::read_to_string(&dir.join(ZINE_FILE))?;
        let mut season_file = toml::from_str::<SeasonFile>(&content).unwrap();

        for article in &mut season_file.articles {
            let markdown = fs::read_to_string(&dir.join(&article.file))?;
            let markdown_parser = MarkdownParser::new_ext(&markdown, Options::all());
            html::push_html(&mut article.html, markdown_parser);
        }
        Ok(season_file.articles)
    }

    fn parse_pages(&self) -> Result<Vec<Page>> {
        let mut pages = vec![];
        let page_dir = self.path.join("pages");
        for entry in WalkDir::new(&page_dir) {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let markdown = fs::read_to_string(path)?;
                let markdown_parser = MarkdownParser::new_ext(&markdown, Options::all());
                let mut html = String::new();
                html::push_html(&mut html, markdown_parser);
                pages.push(Page {
                    html,
                    file_path: path.strip_prefix(&page_dir)?.to_owned(),
                });
            }
        }
        Ok(pages)
    }
}
