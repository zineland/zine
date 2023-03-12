use std::collections::HashMap;

use anyhow::{bail, Result};
use tokio::{runtime::Handle, task};

mod author;
mod callout;
mod inline_link;
mod quote;
mod url_preview;

use crate::data::{self, PreviewEvent};
pub use author::AuthorCode;
pub use inline_link::InlineLink;
use url_preview::{UrlPreviewBlock, UrlPreviewError};

use self::{callout::CalloutBlock, quote::QuoteBlock};

pub trait CodeBlock {
    fn render(&self) -> Result<String>;
}

pub const CALLOUT: &str = "callout";
pub const QUOTE: &str = "quote";
pub const URL_PREVIEW: &str = "urlpreview";

const ALL_CODE_BLOCKS: &[&str] = &[CALLOUT, QUOTE, URL_PREVIEW];

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Fenced<'a> {
    pub name: &'a str,
    pub options: HashMap<String, &'a str>,
}

impl<'a> Fenced<'a> {
    // Empty Fenced.
    fn empty() -> Self {
        Self::default()
    }

    pub fn is_custom_code_block(&self) -> bool {
        ALL_CODE_BLOCKS.contains(&self.name)
    }

    /// Render code block. Return rendered HTML string if success,
    /// otherwise return URL preview error HTML string to remind user we have error.
    ///
    /// If the fenced is unsupported, we simply return `None`.
    pub fn render_code_block(self, block: &'a str) -> Option<String> {
        match self.name {
            URL_PREVIEW => {
                let url = block.trim();

                // Block in place to execute async task
                task::block_in_place(|| {
                    Handle::current().block_on(async {
                        let (first_preview, mut rx) = {
                            // parking_lot RwLock guard isn't async-aware,
                            // we should keep this guard drop in this scope.
                            let data = data::read();
                            if let Some(info) = data.get_preview(url) {
                                let html = UrlPreviewBlock::new(self.options, url, info)
                                    .render()
                                    .unwrap();
                                return Some(html);
                            }

                            data.preview_url(url)
                        };
                        rx.changed()
                            .await
                            .expect("URL preview watch channel receive failed.");
                        let event = rx.borrow();
                        match event.to_owned().expect("Url preview didn't initialized.") {
                            PreviewEvent::Finished(info) => {
                                let html = UrlPreviewBlock::new(self.options, url, info)
                                    .render()
                                    .unwrap();
                                if first_preview {
                                    println!("URL previewed: {url}");
                                }
                                Some(html)
                            }
                            PreviewEvent::Failed(err) => {
                                // Return a preview error block.
                                Some(UrlPreviewError(url, &err).render().unwrap())
                            }
                        }
                    })
                })
            }
            CALLOUT => {
                let html = CalloutBlock::new(self.options, block).render().unwrap();
                Some(html)
            }
            QUOTE => {
                let html = QuoteBlock::parse(block).unwrap().render().unwrap();
                Some(html)
            }
            _ => None,
        }
    }

    pub fn parse(input: &'a str) -> Result<Self> {
        let input = input.trim_end_matches(',');
        if input.is_empty() {
            return Ok(Self::empty());
        }

        let mut raw = input.split(',');
        match raw.next() {
            Some(name) if !name.is_empty() => {
                let options = raw
                    .into_iter()
                    .filter_map(|pair| {
                        let mut v = pair.split(':').take(2);
                        match (v.next(), v.next()) {
                            (Some(key), Some(value)) => {
                                // Replace key's dash to underscore.
                                Some((key.trim().replace('-', "_"), value.trim()))
                            }
                            _ => {
                                println!("Warning: invalid fenced options: {}", pair);
                                None
                            }
                        }
                    })
                    .collect::<HashMap<_, _>>();
                Ok(Fenced { name, options })
            }
            _ => {
                bail!("Invalid fenced: {}", input)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fenced_parsing() {
        assert_eq!(Fenced::parse("").unwrap(), Fenced::empty());
        assert_eq!(Fenced::parse(",").unwrap(), Fenced::empty());

        let fenced = Fenced::parse("callout,").unwrap();
        assert_eq!(fenced.name, "callout");
        assert_eq!(fenced.options, HashMap::default());

        let fenced = Fenced::parse("callout, bg_color: #123456, border-color: #abcdef").unwrap();
        assert_eq!(fenced.name, "callout");

        let options = fenced.options;
        assert_eq!(options["bg_color"], "#123456");
        assert_eq!(options["border_color"], "#abcdef");

        let fenced = Fenced::parse("callout, bg_color #123456, border-color: #abcdef").unwrap();
        assert!(fenced.is_custom_code_block());
        assert_eq!(fenced.name, "callout");

        let options = fenced.options;
        assert_eq!(options.get("bg_color"), None);
        assert_eq!(options["border_color"], "#abcdef");

        let fenced = Fenced::parse("rust").unwrap();
        assert!(!fenced.is_custom_code_block());
    }
}
