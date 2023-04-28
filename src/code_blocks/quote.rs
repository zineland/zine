use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuoteBlock {
    // The author name.
    // Plain text format.
    pub author: Option<String>,
    /// The avatar url.
    pub avatar: Option<String>,
    // The profile of the author.
    // Markdown format.
    pub bio: Option<String>,
    // The comment content.
    // Markdown format.
    pub content: String,
}

impl QuoteBlock {
    pub fn parse(block: &str) -> Result<Self> {
        match toml::from_str::<Self>(block) {
            Ok(quote_block) => Ok(quote_block),
            // Parse failed if the block has invalid toml syntax.
            Err(error) => Err(anyhow!("Parse quote block error: {}", error)),
        }
    }
}
