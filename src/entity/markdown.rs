use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "snake_case"))]
pub struct MarkdownConfig {
    #[serde(default = "MarkdownConfig::default_highlight_code")]
    pub highlight_code: bool,
    #[serde(default = "MarkdownConfig::default_highlight_theme")]
    pub highlight_theme: String,
}

impl minijinja::value::Object for MarkdownConfig {}

impl Display for MarkdownConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MarkdownConfig {{ highlight_code: {}, highlight_theme: {} }}",
            self.highlight_code, self.highlight_theme
        )
    }
}

impl Default for MarkdownConfig {
    fn default() -> Self {
        Self {
            highlight_code: true,
            highlight_theme: Self::default_highlight_theme(),
        }
    }
}

impl MarkdownConfig {
    const DEFAULT_HIGHLIGHT_THEME: &'static str = "monokai";

    fn default_highlight_theme() -> String {
        Self::DEFAULT_HIGHLIGHT_THEME.to_string()
    }

    fn default_highlight_code() -> bool {
        true
    }
}
