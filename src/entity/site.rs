use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::path::Path;
use toml;

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
    /// Writes the site TOML file in the root of the Zine magazine.
    /// This should only be called when creating a new Zine magazine
    pub(crate) fn write_toml(&self, path: &Path) -> Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;

        let toml_str = toml::to_string(&self)?;

        file.write_all("\n[site]\n".as_bytes())?;
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
