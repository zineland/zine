use genkit::{CodeBlock, MarkdownVisitor};

use crate::{
    code_blocks::{AuthorCode, InlineLink},
    data,
};

#[derive(Debug, Clone)]
pub struct ZineMarkdownVisitor;

impl MarkdownVisitor for ZineMarkdownVisitor {
    fn visit_code(&self, code: &str) -> Option<String> {
        if let Some(maybe_author_id) = code.strip_prefix('@') {
            let data = data::read();
            if let Some(author) = data.get_author_by_id(maybe_author_id) {
                // Render author code UI.
                let html = AuthorCode(author)
                    .render()
                    .expect("Render author code failed.");
                return Some(html);
            }
        } else if code.starts_with('/') {
            let data = data::read();
            if let Some(article) = data.get_article_by_path(code.as_ref()) {
                let html = InlineLink::new(&article.title, code, article.cover.as_ref())
                    .render()
                    .expect("Render inline linke failed.");
                return Some(html);
            }
        } else if let Some(topic) = code.strip_prefix('#') {
            let data = data::read();
            if data.is_valid_topic(topic) {
                let html = format!(r#"<a href="/topic/{topic}">#{topic}</a>"#);
                return Some(html);
            }
        }

        None
    }
}
