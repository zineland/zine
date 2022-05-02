use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all(deserialize = "snake_case"))]
pub struct MarkdownConfig {
    #[serde(default = "MarkdownConfig::default_highlight_theme")]
    pub highlight_theme: String,
}

impl Default for MarkdownConfig {
    fn default() -> Self {
        Self {
            highlight_theme: Self::default_highlight_theme(),
        }
    }
}

impl MarkdownConfig {
    const DEFAULT_HIGHLIGHT_THEME: &'static str = "monokai";

    fn default_highlight_theme() -> String {
        Self::DEFAULT_HIGHLIGHT_THEME.to_string()
    }
}
