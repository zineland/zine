use std::{borrow::Cow, fs, path::Path};

use anyhow::{Context as _, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tera::Context;
use time::Date;

use crate::{current_mode, engine, markdown, meta::Meta, Mode};

use super::{AuthorId, EndMatter, Entity};

/// The Meta info of Article.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaArticle {
    pub file: String,
    /// The slug after this artcile rendered.
    /// Default to file name if no slug specified.
    pub slug: Option<String>,
    pub title: String,
    /// The author id of this article.
    /// An article can has zero, one or multiple authors.
    pub author: Option<AuthorId>,
    pub cover: Option<String>,
    /// The publish date. Format like YYYY-MM-dd.
    #[serde(with = "crate::helpers::serde_date")]
    pub pub_date: Date,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Article {
    #[serde(flatten)]
    pub meta: MetaArticle,
    /// The article's markdown content.
    #[serde(default)]
    pub markdown: String,
    /// The optional end matter of the article.
    pub end_matter: Option<EndMatter>,
    /// Whether the article is an featured article.
    /// Featured article will display in home page.
    #[serde(default)]
    pub featured: bool,
    /// Whether publish the article. Publish means generate the article HTML file.
    /// This field would be ignored if in `zine serve` mode, that's mean we alwasy
    /// generate HTML file in this mode.
    #[serde(default)]
    pub publish: bool,
}

impl std::fmt::Debug for Article {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Article")
            .field("meta", &self.meta)
            .field("publish", &self.publish)
            .finish()
    }
}

impl MetaArticle {
    #[inline]
    pub fn slug(&self) -> String {
        self.slug
            .as_ref()
            .cloned()
            .unwrap_or_else(|| self.file.replace(".md", ""))
    }
}

impl Article {
    /// Check whether `author` name is the author of this article.
    pub fn is_author(&self, author: &str) -> bool {
        self.meta
            .author
            .as_ref()
            .map(|inner| inner.is_author(author))
            .unwrap_or_default()
    }

    /// Check whether the article need publish.
    ///
    /// The article need publish in any of two conditions:
    /// - the publish property is true
    /// - in `zine serve` mode
    pub fn need_publish(&self) -> bool {
        self.publish || matches!(current_mode(), Mode::Serve)
    }

    pub fn slug(&self) -> String {
        self.meta.slug()
    }
}

impl Entity for Article {
    fn parse(&mut self, source: &Path) -> Result<()> {
        let file_path = source.join(&self.meta.file);
        let markdown = fs::read_to_string(&file_path).with_context(|| {
            format!("Failed to read markdown file of `{}`", file_path.display())
        })?;
        let (content, end_matter) = split_article_content(&markdown)?;

        // Fallback to file name if no slug specified.
        if self.meta.slug.is_none() {
            self.meta.slug = Some(self.meta.file.replace(".md", ""))
        }
        // Fallback to the default placeholder image if the cover is missing.
        if self.meta.cover.is_none()
            || self.meta.cover.as_ref().map(|cover| cover.is_empty()) == Some(true)
        {
            self.meta.cover = Some(String::from("/static/zine-placeholder.svg"));
        }

        self.markdown = content.to_owned();
        self.end_matter = end_matter;
        Ok(())
    }

    fn render(&self, mut context: Context, dest: &Path) -> Result<()> {
        context.insert(
            "meta",
            &Meta {
                title: Cow::Borrowed(&self.meta.title),
                description: Cow::Owned(markdown::extract_description(&self.markdown)),
                url: Some(Cow::Owned(self.slug())),
                image: self.meta.cover.as_deref().map(Cow::Borrowed),
            },
        );
        context.insert("page_type", "article");
        context.insert("article", &self);
        context.insert("end_matter", &self.end_matter);
        engine::render("article.jinja", &context, dest)?;
        Ok(())
    }
}

static END_MATTER_REGEX: Lazy<Regex> = Lazy::new(|| {
    // The regex is an variant of zola's fronmatter regex.
    Regex::new(
        r"[[:space:]]*(?:$|(?:\r?\n((?s).*(?-s))))[[:space:]]*\+\+\+(\r?\n(?s).*?(?-s))\+\+\+[[:space:]]*$",
    )
    .unwrap()
});

// Splite article content and optional end matter from article markdown.
fn split_article_content(markdown: &str) -> Result<(&str, Option<EndMatter>)> {
    if let Some(caps) = END_MATTER_REGEX.captures(markdown) {
        // caps[0] is the full match
        // caps[1] => article
        // caps[2] => end matter
        let article = caps.get(1).expect("").as_str().trim();
        let end_matter = caps.get(2).expect("").as_str().trim();
        match toml::from_str::<EndMatter>(end_matter) {
            Ok(end_matter) => {
                return Ok((article, Some(end_matter)));
            }
            // Parse failed if the end matter has invalid toml syntax.
            Err(error) => println!("Parse end matter error: {}", error),
        }
    }

    Ok((markdown, None))
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::split_article_content;

    #[test_case(r#"
    Hello
    "#; "No end matter")]
    #[test_case(r#"
    Hello
    +++
    "#; "Invalid end matter")]
    #[test_case(r#"
    Hello
    +++
    +++
    "#; "Empty end matter")]
    fn test_parse_end_matter_none(input: &str) {
        let r = split_article_content(input).unwrap();
        assert!(r.1.is_none());
    }

    #[test_case(r#"
    Hello
    +++
    [[abc]]
    +++
    "#; "Invalid end matter1")]
    #[test_case(r#"
    Hello
    +++
    [[comment]]
    xxx = "yyy"
    +++
    "#; "Invalid end matter2")]
    #[test_case(r#"
    Hello
    +++
    [[comment]]
    author = 123
    content = 123
    +++
    "#; "Invalid end matter3")]
    fn test_parse_end_matter_invalid(input: &str) {
        let (_, end_matter) = split_article_content(input).unwrap();
        assert!(end_matter.is_none());
    }

    #[test_case(r#"
    Hello
    +++
    [[comment]]
    author = "Alice"
    content = "Hi"
    +++
    "#; "Normal end matter")]
    fn test_parse_end_matter_normal(input: &str) {
        let (_, end_matter) = split_article_content(input).unwrap();
        let end_matter = end_matter.unwrap();
        assert_eq!(1, end_matter.comments.len());
        let comment = end_matter.comments.get(0).unwrap();
        assert_eq!("Alice", comment.author);
        assert_eq!(None, comment.bio);
        assert_eq!("Hi", comment.content);
    }

    #[test_case(r#"
    Hello
    +++
    [[comment]]
    author = "Alice"
    bio = "Developer"
    content = "Hi"
    +++
    "#; "Single comment in end matter")]
    fn test_parse_end_matter_full(input: &str) {
        let (_, end_matter) = split_article_content(input).unwrap();
        let end_matter = end_matter.unwrap();
        assert_eq!(1, end_matter.comments.len());
        let comment = end_matter.comments.get(0).unwrap();
        assert_eq!("Alice", comment.author);
        assert_eq!(Some("Developer".into()), comment.bio);
        assert_eq!("Hi", comment.content);
    }

    #[test_case(r#"
    Hello

    +++
    [[comment]]
    author = "Alice"
    content = "Hi"

    [[comment]]
    author = "Bob"
    bio = "Rustacean"
    content = "Hey"
    +++
    "#; "Multipe comments in end matter")]
    fn test_parse_end_matter_multiple(input: &str) {
        let (_, end_matter) = split_article_content(input).unwrap();
        let end_matter = end_matter.unwrap();
        let mut iter = end_matter.comments.iter();
        assert_eq!(2, iter.len());

        let comment = iter.next().unwrap();
        assert_eq!("Alice", comment.author);
        assert_eq!(None, comment.bio);
        assert_eq!("Hi", comment.content);

        let comment = iter.next().unwrap();
        assert_eq!("Bob", comment.author);
        assert_eq!(Some("Rustacean".into()), comment.bio);
        assert_eq!("Hey", comment.content);
    }
}
