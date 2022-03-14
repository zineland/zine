use serde::{Deserialize, Serialize};

/// The end matter below the article content.
/// 
/// Here is the format:
/// ```toml
/// +++
/// [comment]
/// author = "Bob"
/// bio = "Rustaceans"
/// content = "Have a good day!"
/// +++
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct EndMatter {
    #[serde(rename(deserialize = "comment"))]
    pub comments: Vec<Comment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Comment {
    // The author name. 
    // Plain text format.
    pub author: String,
    // The profile of the author. 
    // Markdown format.
    pub bio: Option<String>,
    // The comment content. 
    // Markdown format.
    pub content: String,
}
