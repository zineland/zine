use anyhow::Result;

mod author;
mod fenced;
mod highlight;
mod url_preview;

use crate::{data, helpers, html};
pub use author::AuthorCode;
use fenced::Fenced;
use url_preview::{UrlPreviewBlock, UrlPreviewError};

use self::highlight::HighlightBlock;

pub trait CodeBlock {
    fn render(&self) -> Result<String>;
}

const HIGHTLIGHT: &str = "highlight";
const URL_PREVIEW: &str = "urlpreview";

const ALL_CODE_BLOCKS: &[&str] = &[HIGHTLIGHT, URL_PREVIEW];

pub fn is_custom_code_block(fenced: &str) -> bool {
    ALL_CODE_BLOCKS.contains(&fenced)
}

/// Render code block. Return rendered HTML string if success,
/// otherwise return URL preview error HTML string to remind user we have error.
///
/// If the fenced is unsupported, we simply return `None`.
pub async fn render_code_block(fenced: &str, block: &str) -> Option<String> {
    let fenced = Fenced::parse(fenced).ok()?;
    match fenced.name {
        URL_PREVIEW => {
            let url = block.trim();

            {
                // parking_lot Mutex guard isn't async-aware,
                // we should keep this guard drop in this scope.
                let data = data::read();
                if let Some((title, description)) = data.url_previews().get(url) {
                    return Some(UrlPreviewBlock(url, title, description).render().unwrap());
                }
            }

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
        HIGHTLIGHT => {
            let html = HighlightBlock::new(&fenced.options, block)
                .render()
                .unwrap();
            Some(html)
        }
        _ => None,
    }
}
