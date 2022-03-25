use std::borrow::Cow;

use serde::Serialize;

use crate::strip_markdown::strip_markdown;

/// The meta info of the HTML page.
#[derive(Debug, Default, Serialize)]
pub struct Meta<'a> {
    pub title: Cow<'a, str>,
    pub description: Cow<'a, str>,
    pub url: Option<Cow<'a, str>>,
    pub image: Option<Cow<'a, str>>,
}

impl<'a> Meta<'a> {
    pub fn is_filled(&self) -> bool {
        !self.title.is_empty() && !self.description.is_empty()
    }

    pub fn truncate(&mut self) {
        self.title.to_mut().truncate(200);
        self.description.to_mut().truncate(200);
    }
}

/// Extract the description from markdown content.
///
/// The strategy is extract the first meaningful line,
/// and only take at most 200 plain chars from this line.
pub fn extract_description_from_markdown(markdown: &str) -> String {
    markdown
        .lines()
        .find_map(|line| {
            // Ignore heading, image line.
            let line = line.trim();
            if line.is_empty() || line.starts_with(&['#', '!']) {
                None
            } else {
                let raw = strip_markdown(line);
                // If the stripped raw text is empty, we step to next one.
                if raw == "\n" || raw.is_empty() {
                    None
                } else {
                    // No more than 200 chars.
                    // Also, replace double quote to single quote.
                    Some(raw.chars().take(200).collect::<String>().replace('"', "'"))
                }
            }
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::iter;

    use super::extract_description_from_markdown;
    use test_case::test_case;

    #[test_case("aaaa"; "case1")]
    fn test_extract_decription_from_markdown1(markdown: &str) {
        assert_eq!("aaaa", extract_description_from_markdown(markdown));
    }

    #[test_case("

    aaaa"; "case0")]
    #[test_case("
    # h1
    aaaa"; "case1")]
    #[test_case("
    ![](img.png)
    aaaa"; "case2")]
    fn test_extract_decription_from_markdown2(markdown: &str) {
        assert_eq!("aaaa", extract_description_from_markdown(markdown));
    }

    #[test_case("a \"aa\" a"; "quote replace")]
    fn test_extract_decription_from_markdown3(markdown: &str) {
        assert_eq!("a 'aa' a", extract_description_from_markdown(markdown));
    }

    #[test]
    fn test_extract_decription_from_markdown_at_most_1_paragraphs() {
        let base = iter::repeat('a').take(10).collect::<String>();
        let mut p1 = base.clone();
        p1.push('\n');
        p1.push_str(&base.clone());
        assert_eq!(base, extract_description_from_markdown(&p1));
    }

    #[test]
    fn test_extract_decription_from_markdown_at_most_200_chars() {
        let p1 = iter::repeat('a').take(400).collect::<String>();

        let p2 = p1.clone();
        // Never extract more than 200 chars.
        assert_eq!(p1[..200], extract_description_from_markdown(&p2));
    }
}
