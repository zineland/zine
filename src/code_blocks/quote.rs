use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tera::Context;

use crate::engine;

use super::CodeBlock;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuoteBlock {
    // The author name.
    // Plain text format.
    pub author: Option<String>,
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

impl CodeBlock for QuoteBlock {
    fn render(&self) -> Result<String> {
        let mut context = Context::new();
        context.insert("quote", &self);
        let html = engine::get_tera().render("blocks/quote.jinja", &context)?;
        Ok(html)
    }
}
