use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::Entity;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "snake_case"))]
pub struct Theme {
    //whether dark mode is enabled (boolean)
    pub dark_mode: Option<bool>,
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
    // The page color.
    #[serde(default = "Theme::default_page_color")]
    pub page_color: String,
    // The background image url.
    #[serde(default)]
    pub background_image: Option<String>,
    // The extra head template path, will be parsed to html.
    pub head_template: Option<String>,
    // The custom footer template path, will be parsed to html.
    pub footer_template: Option<String>,
    // The extend template path for article page, will be parsed to html.
    // Normally, this template can be a comment widget, such as https://giscus.app.
    pub article_extend_template: Option<String>,
    #[serde(skip_serializing)]
    pub default_cover: Option<String>,
    #[serde(skip_serializing)]
    pub default_avatar: Option<String>,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            dark_mode: Some(false),
            primary_color: Self::default_primary_color(),
            main_color: Self::default_main_color(),
            link_color: Self::default_link_color(),
            secondary_color: Self::default_secondary_color(),
            page_color: Self::default_page_color(),
            background_image: None,
            head_template: None,
            footer_template: None,
            article_extend_template: None,
            default_cover: None,
            default_avatar: None,
        }
    }
}

impl std::fmt::Debug for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Theme")
            .field("dark_mode", &self.dark_mode)
            .field("primary_color", &self.primary_color)
            .field("main_color", &self.main_color)
            .field("link_color", &self.link_color)
            .field("secondary_color", &self.secondary_color)
            .field("page_color", &self.page_color)
            .field("background_image", &self.background_image)
            .field("head_template", &self.head_template.is_some())
            .field("footer_template", &self.footer_template.is_some())
            .field(
                "article_extend_template",
                &self.article_extend_template.is_some(),
            )
            .field("default_cover", &self.default_cover)
            .field("default_avatar", &self.default_avatar)
            .finish()
    }
}

impl Theme {
    const DEFAULT_PRIMARY_COLOR_LIGHT: &'static str = "#2563eb";
    const DEFAULT_MAIN_COLOR_LIGHT: &'static str = "#ffffff";
    const DEFAULT_LINK_COLOR_LIGHT: &'static str = "#2563eb";
    const DEFAULT_SECONDARY_COLOR_LIGHT: &'static str = "#eff3f7";
    const DEFAULT_PAGE_COLOR_LIGHT: &'static str = "#ffffff";

    const DEFAULT_PRIMARY_COLOR_DARK: &'static str = "#0d0d0d";
    // const DEFAULT_MAIN_COLOR_DARK: &'static str = "#ffffff";
    // const DEFAULT_LINK_COLOR_DARK: &'static str = "#2563eb";
    const DEFAULT_SECONDARY_COLOR_DARK: &'static str = "#303030";
    const DEFAULT_PAGE_COLOR_DARK: &'static str = "#505050";

    fn default_primary_color() -> String {
        Self::DEFAULT_PRIMARY_COLOR_LIGHT.to_string()
    }

    fn default_main_color() -> String {
        Self::DEFAULT_MAIN_COLOR_LIGHT.to_string()
    }

    fn default_link_color() -> String {
        Self::DEFAULT_LINK_COLOR_LIGHT.to_string()
    }

    fn default_secondary_color() -> String {
        Self::DEFAULT_SECONDARY_COLOR_LIGHT.to_string()
    }

    fn default_page_color() -> String {
        Self::DEFAULT_PAGE_COLOR_LIGHT.to_string()
    }

    fn default_page_color_dark() -> String {
        Self::DEFAULT_PAGE_COLOR_DARK.to_string()
    }

    fn default_primary_color_dark() -> String {
        Self::DEFAULT_PRIMARY_COLOR_DARK.to_string()
    }

    fn default_secondary_color_dark() -> String {
        Self::DEFAULT_SECONDARY_COLOR_DARK.to_string()
    }

    fn change_defaults(&mut self) {
        if self.dark_mode.unwrap_or(false) {
            if self.page_color == Self::default_page_color() {
                self.page_color = Self::default_page_color_dark(); // Changing page colour, if dark theme is enabled
            }
            if self.primary_color == Self::default_primary_color() {
                self.primary_color = Self::default_primary_color_dark(); // Changing primary colour, if dark theme is enabled
            }
            if self.secondary_color == Self::default_secondary_color() {
                self.secondary_color = Self::default_secondary_color_dark(); // Changing secondary colour, if dark theme is enabled
            }
        }
    }
}

impl Entity for Theme {
    fn parse(&mut self, source: &Path) -> Result<()> {
        self.change_defaults(); // Change default colors if dark mode is enabled.

        if self.default_cover.is_none() {
            self.default_cover = Some(String::from("/static/zine-placeholder.svg"));
        }
        if self.default_avatar.is_none() {
            self.default_avatar = Some(String::from("/static/zine.png"));
        }

        if let Some(head_template) = self.head_template.as_ref() {
            // Read head template from path to html.
            self.head_template = Some(
                fs::read_to_string(source.join(head_template)).with_context(|| {
                    format!(
                        "Failed to parse the head template: `{}`",
                        source.join(head_template).display(),
                    )
                })?,
            );
        }
        if let Some(footer_template) = self.footer_template.as_ref() {
            // Read footer template from path to html.
            self.footer_template = Some(
                fs::read_to_string(source.join(footer_template)).with_context(|| {
                    format!(
                        "Failed to parse the footer template: `{}`",
                        source.join(footer_template).display(),
                    )
                })?,
            );
        }
        if let Some(article_extend_template) = self.article_extend_template.as_ref() {
            // Read article extend template from path to html.
            self.article_extend_template = Some(
                fs::read_to_string(source.join(article_extend_template)).with_context(|| {
                    format!(
                        "Failed to parse the article extend template: `{}`",
                        source.join(article_extend_template).display(),
                    )
                })?,
            );
        }
        Ok(())
    }
}
