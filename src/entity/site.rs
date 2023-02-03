use crate::{Article, Issue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::{
    env,
    path::{Path, PathBuf},
};
use toml;

use crate::{helpers::run_command, ZINE_FILE};
#[derive(Default)]
pub struct SiteBuilder {
    name: Option<String>,
    source: PathBuf,
    author: String,
    site: Site,
}

impl SiteBuilder {
    // Defines a new site with default settings while providing a new an optional site `name`
    pub fn new(name: Option<String>) -> Result<Self> {
        let source = if let Some(name) = name.as_ref() {
            env::current_dir()?.join(name)
        } else {
            env::current_dir()?
        };
        let site_name = name.clone();
        Ok(Self {
            name,
            source: source,
            site: Site {
                name: site_name.unwrap_or_default(),
                ..Site::default()
            },
            ..Default::default()
        })
    }
    pub fn create_new_zine_magazine(&mut self) -> Result<()> {
        // Create Root of Zine Magazine
        if !self.source.exists() {
            std::fs::create_dir_all(&self.source)?;
        }
        if !self.source.join(crate::ZINE_FILE).exists() {

            // This requires that we add the author struct to Site
            //let author = run_command("git", &["config", "user.name"])
            //    .ok()
            //    .unwrap_or_default();
            //self.author = author;
            let content_dir = &self.source.join(crate::ZINE_CONTENT_DIR);
            self.site.write_toml(&self.source.join(crate::ZINE_FILE))?;
            std::fs::create_dir_all(&content_dir)?;

            let mut issue = Issue::new();
            issue.set_title("issue").set_issue_number(1);
            issue = issue.finalize();
            issue.create_issue_dir(&content_dir)?;

            let article = Article::default();
            issue.articles.push(article);

            let issue_dir = &content_dir.join(&issue.dir);
            let toml_file = &issue_dir
                .join(crate::ZINE_FILE);
            // Write Issue zine.toml
            issue.write_new_issue(&content_dir)?;

            for article in &issue.articles {
                article.append_article_to_toml(&toml_file)?;
            }

            if !issue.articles.is_empty() {
                issue.write_initial_markdown_file(&content_dir)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod site_builder {

    use super::Site;
    use super::SiteBuilder;
    use tempfile::tempdir;

    #[test]
    fn site_to_build() {
        let temp_dir = tempdir().unwrap();
        let site = SiteBuilder::default();
        assert_eq!(site.name, None);
        assert_eq!(site.author, "");
        assert_eq!(site.site.url, "http://localhost");

        let file_path = temp_dir.path().join("dummy.toml");
        assert!(site.site.write_toml(&file_path.as_path()).is_ok());

        let read_contents = std::fs::read_to_string(&file_path).unwrap();
        let data: Site = toml::from_str(&read_contents).unwrap();

        assert_eq!(data.name, "My New Magazine Powered by Rust!");
        assert_eq!(data.url, "http://localhost");

        drop(file_path);
        assert!(temp_dir.close().is_ok());
    }

    #[test]
    fn test_site_builder_new() {
        let new_site = SiteBuilder::new(Some("test".to_string()));
        let temp_dir = tempdir().unwrap();

        if let Ok(new_site) = new_site {
            let file_path = temp_dir.path().join("dummy.toml");
            assert_eq!(new_site.name, Some("test".to_string()));
            assert_eq!(new_site.site.name, "test".to_string());
            assert!(new_site.site.write_toml(&file_path).is_ok());

            let read_contents = std::fs::read_to_string(&file_path).unwrap();
            let data: Site = toml::from_str(&read_contents).unwrap();

            assert_eq!(data.name, "test");
            assert_eq!(data.url, "http://localhost");

            drop(file_path);
            assert!(temp_dir.close().is_ok());
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Site {
    /// The absolute url of this site.
    pub url: String,
    pub cdn: Option<String>,
    pub name: String,
    pub description: Option<String>,
    /// The repository edit url of this zine website.
    pub edit_url: Option<String>,
    /// The OpenGraph social image.
    pub social_image: Option<String>,
    /// The locale to localize some builtin text.
    /// Default to 'en'.
    #[serde(default = "default_locale")]
    pub locale: String,
    #[serde(rename(deserialize = "menu"))]
    #[serde(default)]
    pub menus: Vec<Menu>,
}

impl Default for Site {
    fn default() -> Self {
        Self {
            url: "http://localhost".into(),
            cdn: None,
            name: "My New Magazine Powered by Rust!".into(),
            description: None,
            edit_url: None,
            social_image: None,
            locale: "en".into(),
            menus: vec![],
        }
    }
}

impl Site {
    fn write_toml(&self, path: &Path) -> Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;

        let toml_str = toml::to_string(&self)?;

        file.write_all(toml_str.as_bytes())?;

        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Menu {
    pub name: String,
    pub url: String,
}

fn default_locale() -> String {
    "en".to_owned()
}
