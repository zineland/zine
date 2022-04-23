use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Site {
    /// The absolute url of this site.
    pub url: String,
    pub name: String,
    pub logo: Option<String>,
    pub description: Option<String>,
    /// The OpenGraph social image.
    pub social_image: Option<String>,
    /// The locale to localize some builtin text.
    /// Default to 'en'.
    pub locale: Option<String>,
    #[serde(rename(deserialize = "menu"))]
    #[serde(default)]
    pub menus: Vec<Menu>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Menu {
    pub name: String,
    pub url: String,
}
