use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all(deserialize = "snake_case"))]
pub struct MarkdownConfig {
    pub highlight_theme: String,
}

impl Default for MarkdownConfig {
    fn default() -> Self {
        Self {
            highlight_theme: String::from("monokai"),
        }
    }
}
