use std::collections::HashMap;

use anyhow::{bail, Result};

mod author;
mod callout;
mod inline_link;
mod quote;
pub mod url_preview;

pub use author::AuthorCode;
pub use inline_link::InlineLink;

pub use self::{callout::CalloutBlock, quote::QuoteBlock};

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

    pub fn parse(input: &'a str) -> Result<Self> {
        let input = input.trim_end_matches(',');
        if input.is_empty() {
            return Ok(Self::empty());
        }

        let mut raw = input.split(',');
        match raw.next() {
            Some(name) if !name.is_empty() => {
                let options = raw
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
