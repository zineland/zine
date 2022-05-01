use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all(deserialize = "snake_case"))]
pub struct MarkdownConfig {
<<<<<<< HEAD
    #[serde(default = "MarkdownConfig::default_highlight_code")]
    pub highlight_code: bool,
    #[serde(default = "MarkdownConfig::default_highlight_theme")]
=======
>>>>>>> Support customize highlight theme
    pub highlight_theme: String,
}

impl Default for MarkdownConfig {
    fn default() -> Self {
        Self {
<<<<<<< HEAD
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
=======
            highlight_theme: String::from("monokai"),
        }
    }
}
>>>>>>> Support customize highlight theme
