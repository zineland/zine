use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "snake_case"))]
pub struct MarkdownConfig {
    #[serde(default = "MarkdownConfig::default_highlight_code")]
    pub highlight_code: bool,
    #[serde(default = "MarkdownConfig::default_highlight_theme")]
    pub highlight_theme: String,
    #[serde(default = "MarkdownConfig::default_pre_code_color")]
    pub pre_code_color: String,
    #[serde(default = "MarkdownConfig::default_pre_bg_color")]
    pub pre_bg_color: String,
}

impl Default for MarkdownConfig {
    fn default() -> Self {
        Self {
            highlight_code: true,
            highlight_theme: Self::default_highlight_theme(),
            pre_code_color: Self::default_pre_code_color(),
            pre_bg_color: Self::default_pre_code_color(),
        }
    }
}

impl MarkdownConfig {
    const DEFAULT_HIGHLIGHT_THEME: &'static str = "monokai";
    const DEFAULT_PRE_CODE_COLOR: &'static str = "#61676c";
    const DEFAULT_PRE_BG_COLOR: &'static str = "#f5f5f5";

    fn default_highlight_theme() -> String {
        Self::DEFAULT_HIGHLIGHT_THEME.to_string()
    }

    fn default_pre_code_color() -> String {
        Self::DEFAULT_PRE_CODE_COLOR.to_string()
    }

    fn default_pre_bg_color() -> String {
        Self::DEFAULT_PRE_BG_COLOR.to_string()
    }

    fn default_highlight_code() -> bool {
        true
    }
}
