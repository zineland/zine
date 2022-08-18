use std::collections::HashMap;

use anyhow::{bail, Result};

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Fenced<'a> {
    pub name: &'a str,
    pub options: HashMap<&'a str, &'a str>,
}

impl<'a> Fenced<'a> {
    // Empty Fenced.
    fn empty() -> Self {
        Self::default()
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
                            (Some(key), Some(value)) => Some((key.trim(), value.trim())),
                            _ => {
                                println!("Invalid fenced options: {}", pair);
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

        let fenced = Fenced::parse("highlight,").unwrap();
        assert_eq!(fenced.name, "highlight");
        assert_eq!(fenced.options, HashMap::default());

        let fenced = Fenced::parse("highlight, bg_color: #123456, outline_color: #abcdef").unwrap();
        assert_eq!(fenced.name, "highlight");

        let options = fenced.options;
        assert_eq!(options["bg_color"], "#123456");
        assert_eq!(options["outline_color"], "#abcdef");

        let fenced = Fenced::parse("highlight, bg_color #123456, outline_color: #abcdef").unwrap();
        assert_eq!(fenced.name, "highlight");

        let options = fenced.options;
        assert_eq!(options.get("bg_color"), None);
        assert_eq!(options["outline_color"], "#abcdef");
    }
}
