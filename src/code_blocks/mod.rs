use anyhow::Result;

mod author;
mod url_preview;

use crate::{data, helpers, html};
pub use author::AuthorCode;

use url_preview::{UrlPreviewBlock, UrlPreviewError};

pub trait CodeBlock {
    fn render(&self) -> Result<String>;
}

const URL_PREVIEW: &str = "urlpreview";

const ALL_CODE_BLOCKS: &[&str] = &[URL_PREVIEW];

pub fn is_custom_code_block(fenced: &str) -> bool {
    ALL_CODE_BLOCKS.contains(&fenced)
}

/// Render code block. Return rendered HTML string if success,
/// otherwise return URL preview error HTML string to remind user we have error.
///
/// If the fenced is unsupported, we simply return `None`.
pub async fn render_code_block(fenced: &str, block: &str) -> Option<String> {
    match fenced {
        URL_PREVIEW => {
            let url = block.trim();

            if let Some((title, description)) = data::read().url_previews().get(url) {
                Some(UrlPreviewBlock(url, title, description).render().unwrap())
            } else {
                println!("Preview new url: {}", url);
                match helpers::fetch_url(url).await {
                    Ok(html) => {
                        let meta = html::parse_html_meta(html);
                        let html = UrlPreviewBlock(url, &meta.title, &meta.description)
                            .render()
                            .unwrap();
                        data::write().insert_url_preview(
                            url,
                            (meta.title.into_owned(), meta.description.into_owned()),
                        );
                        Some(html)
                    }
                    // Return a preview error block.
                    Err(err) => Some(UrlPreviewError(url, &err.to_string()).render().unwrap()),
                }
            }
        }
        _ => None,
    }
}
