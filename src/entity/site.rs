use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::{
    env,
    path::{Path, PathBuf},
};
use toml;

use anyhow::Result;

use crate::ZINE_FILE;

#[derive(Default)]
struct SiteBuilder {
    name: Option<String>,
    source: PathBuf,
    author: String,
    site: Site,
}

impl SiteBuilder {
    // Defines a new site with default settings while providing a new an optional site `name`
    fn new(name: Option<String>) -> Result<Self> {
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
                name: site_name.unwrap_or("".to_string()),
                ..Site::default()
            },
            ..Default::default()
        })
    }
    fn create_new_zine_dir(&self) -> Result<PathBuf> {
        if !self.source.exists() {
            std::fs::create_dir_all(&self.source)?
        }
        Ok(self.source.clone())
    }
}

#[cfg(test)]
mod site_builder {

    use super::SiteBuilder;
    use std::env;

    #[test]
    fn site_to_build() {
        let work_space = std::path::Path::new("/tmp");
        assert!(env::set_current_dir(&work_space).is_ok());

        let site = SiteBuilder::default();

        assert_eq!(site.name, None);
        assert_eq!(site.author, "");
        assert_eq!(site.site.url, "http://localhost");
        assert!(site.create_new_zine_dir().is_ok());
        assert!(site.site.write_toml(&site.source).is_ok());
    }

    #[test]
    fn test_site_builder_new() {
        let new_site = SiteBuilder::new(Some("test".to_string()));

        if let Ok(new_site) = new_site {
            assert_eq!(new_site.name, Some("test".to_string()));
            assert_eq!(new_site.site.name, "test".to_string());
            assert!(new_site.create_new_zine_dir().is_ok());
            assert!(new_site.site.write_toml(&new_site.source).is_ok());
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
            .open(path.join(ZINE_FILE))?;

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
