use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Menu {
    pub name: String,
    pub url: String,
}

fn default_locale() -> String {
    "en".to_owned()
}
