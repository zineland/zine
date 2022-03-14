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

impl std::fmt::Debug for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Theme")
            .field("primary_color", &self.primary_color)
            .field("primary_text_color", &self.primary_text_color)
            .field("primary_link_color", &self.primary_link_color)
            .field("secondary_color", &self.secondary_color)
            .field("background_image", &self.background_image)
            .field("footer_template", &self.footer_template.is_some())
            .finish()
    }
}

impl Theme {
    const DEFAULT_PRIMARY_COLOR: &'static str = "#2563eb";
    const DEFAULT_TEXT_COLOR: &'static str = "#ffffff";
    const DEFAULT_LINK_COLOR: &'static str = "#2563eb";
    const DEFAULT_SECONDARY_COLOR: &'static str = "#eff3f7";

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

impl Entity for Theme {
    fn parse(&mut self, source: &Path) -> Result<()> {
        if let Some(footer_template) = self.footer_template.as_ref() {
            // Read footer tempolate from path to html.
            self.footer_template = Some(fs::read_to_string(source.join(&footer_template))?);
        }
        Ok(())
    }
}
