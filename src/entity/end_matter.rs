use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::Entity;

#[derive(Debug, Serialize, Deserialize)]
pub struct EndMatter {
    #[serde(rename(deserialize = "comment"))]
    pub comments: Vec<Comment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Comment {
    pub author: String,
    pub link: Option<String>,
    pub content: String,
}

impl Entity for EndMatter {
    fn parse(&mut self, _source: &Path) -> Result<()> {
        Ok(())
    }
}
