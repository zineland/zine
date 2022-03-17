use std::{fs, path::Path};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::Entity;

#[derive(Serialize, Deserialize)]
#[serde(rename_all(deserialize = "kebab-case"))]
pub struct Theme {
    // The primary color.
    #[serde(default = "Theme::default_primary_color")]
    pub primary_color: String,
    // The text main color.
    #[serde(default = "Theme::default_main_color")]
    pub main_color: String,
    // The article's link color.
    #[serde(default = "Theme::default_link_color")]
    pub link_color: String,
    // The background color.
    #[serde(default = "Theme::default_secondary_color")]
    pub secondary_color: String,
    // The background image url.
    #[serde(default)]
    pub background_image: Option<String>,
    // The custom footer template path, will be parsed to html.
    pub footer_template: Option<String>,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary_color: Self::default_primary_color(),
            main_color: Self::default_main_color(),
            link_color: Self::default_link_color(),
            secondary_color: Self::default_secondary_color(),
            background_image: None,
            footer_template: None,
        }
    }
}

impl std::fmt::Debug for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Theme")
            .field("primary_color", &self.primary_color)
            .field("main_color", &self.main_color)
            .field("link_color", &self.link_color)
            .field("secondary_color", &self.secondary_color)
            .field("background_image", &self.background_image)
            .field("footer_template", &self.footer_template.is_some())
            .finish()
    }
}

impl Theme {
    const DEFAULT_PRIMARY_COLOR: &'static str = "#2563eb";
    const DEFAULT_MAIN_COLOR: &'static str = "#ffffff";
    const DEFAULT_LINK_COLOR: &'static str = "#2563eb";
    const DEFAULT_SECONDARY_COLOR: &'static str = "#eff3f7";

    fn default_primary_color() -> String {
        Self::DEFAULT_PRIMARY_COLOR.to_string()
    }

    fn default_main_color() -> String {
        Self::DEFAULT_MAIN_COLOR.to_string()
    }

    fn default_link_color() -> String {
        Self::DEFAULT_LINK_COLOR.to_string()
    }

    fn default_secondary_color() -> String {
        Self::DEFAULT_SECONDARY_COLOR.to_string()
    }
}

impl Entity for Theme {
    fn parse(&mut self, source: &Path) -> Result<()> {
        if let Some(footer_template) = self.footer_template.as_ref() {
            // Read footer tempolate from path to html.
            self.footer_template = Some(fs::read_to_string(source.join(&footer_template))?);
        }
        Ok(())
    }
}
