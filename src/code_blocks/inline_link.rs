use crate::Entity;

use super::CodeBlock;

pub struct InlineLink {
    title: String,
    url: String,
    image: String,
}

impl CodeBlock for InlineLink {
    fn render(&self) -> anyhow::Result<String> {
        todo!()
    }
}
