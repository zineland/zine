use serde::{Deserialize, Serialize};

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
