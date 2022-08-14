use std::{borrow::Cow, path::Path};

use anyhow::Result;
use serde::{de, ser::SerializeSeq, Deserialize, Serialize};
use tera::Context;

use crate::{engine, markdown, meta::Meta, Entity};

/// AuthorName represents a single author or multiple co-authors.
#[derive(Debug, Clone)]
pub enum AuthorName {
    // Single author.
    One(String),
    // Co-authors.
    List(Vec<String>),
}

/// The author of an article. Declared in the root `zine.toml`'s **[authors]** table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    /// The author id.
    #[serde(skip_deserializing, default)]
    pub id: String,
    /// The author's name. Will fallback to capitalized id if missing.
    pub name: Option<String>,
    /// The optional avatar url. Will fallback to default zine logo if missing.
    pub avatar: Option<String>,
    /// The bio of author (markdown format).
    pub bio: String,
    /// Whether the author is an editor.
    #[serde(default)]
    #[serde(rename(deserialize = "editor"))]
    pub is_editor: bool,
}

// A [`Author`] struct with additional `article_count` field.
#[derive(Debug, Serialize)]
struct AuthorExt<'a> {
    #[serde(flatten)]
    author: &'a Author,
    // How many articles this author has.
    article_count: usize,
}

#[derive(Default, Serialize)]
pub struct AuthorList<'a> {
    authors: Vec<AuthorExt<'a>>,
}

impl AuthorName {
    pub fn is_author(&self, name: &str) -> bool {
        match self {
            Self::One(author) => author.eq_ignore_ascii_case(name),
            Self::List(authors) => authors.iter().any(|a| a.eq_ignore_ascii_case(name)),
        }
    }
}

impl<'a> AuthorList<'a> {
    pub fn record_author(&mut self, author: &'a Author, article_count: usize) {
        self.authors.push(AuthorExt {
            author,
            article_count,
        });
    }

    fn render_title(&self) -> Result<String> {
        engine::render_str(r#"{{ fluent(key="author-list") }}"#, &Context::new())
    }
}

impl Entity for Author {
    fn parse(&mut self, _source: &Path) -> anyhow::Result<()> {
        // Fallback to default zine avatar if neccessary.
        if self.avatar.is_none()
            || self.avatar.as_ref().map(|avatar| avatar.is_empty()) == Some(true)
        {
            self.avatar = Some(String::from("/static/zine.png"));
        }
        Ok(())
    }

    fn render(&self, mut context: Context, dest: &Path) -> anyhow::Result<()> {
        let slug = format!("@{}", self.id.to_lowercase());
        context.insert(
            "meta",
            &Meta {
                title: Cow::Borrowed(self.name.as_deref().unwrap_or(&self.id)),
                description: Cow::Owned(markdown::extract_description(&self.bio)),
                url: Some(Cow::Borrowed(&slug)),
                image: None,
            },
        );
        context.insert("author", &self);
        engine::render("author.jinja", &context, dest.join(slug))?;
        Ok(())
    }
}

impl<'a> Entity for AuthorList<'a> {
    fn render(&self, mut context: Context, dest: &Path) -> anyhow::Result<()> {
        context.insert(
            "meta",
            &Meta {
                title: Cow::Owned(self.render_title()?),
                description: Cow::Owned(String::new()),
                url: Some(Cow::Borrowed("authors")),
                image: None,
            },
        );
        context.insert("authors", &self.authors);
        engine::render("author-list.jinja", &context, dest.join("authors"))?;
        Ok(())
    }
}

impl<'de> Deserialize<'de> for AuthorName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(AuthorNameVisitor)
    }
}

impl Serialize for AuthorName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            AuthorName::One(author) => serializer.serialize_str(author),
            AuthorName::List(authors) => {
                let mut seq = serializer.serialize_seq(Some(authors.len()))?;
                for author in authors {
                    seq.serialize_element(author)?;
                }
                seq.end()
            }
        }
    }
}

struct AuthorNameVisitor;

impl<'de> de::Visitor<'de> for AuthorNameVisitor {
    type Value = AuthorName;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("plain string or string list")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(AuthorName::One(v.to_string()))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut authors = Vec::new();
        while let Some(author) = seq.next_element()? {
            // Avoid author duplication.
            if !authors.contains(&author) {
                authors.push(author);
            }
        }
        Ok(AuthorName::List(authors))
    }
}

#[cfg(test)]
mod tests {

    use super::AuthorName;
    #[test]
    fn test_author_name() {
        assert!(matches!(
            serde_json::from_str::<AuthorName>("\"Alice\"").unwrap(),
            AuthorName::One(name) if name == String::from("Alice"),
        ));
        assert!(matches!(
            serde_json::from_str::<AuthorName>("[\"Alice\",\"Bob\"]").unwrap(),
            AuthorName::List(names) if names == vec![String::from("Alice"), String::from("Bob")],
        ));
        assert!(matches!(
            serde_json::from_str::<AuthorName>("[\"Alice\",\"Bob\", \"Alice\"]").unwrap(),
            AuthorName::List(names) if names == vec![String::from("Alice"), String::from("Bob")],
        ));
        assert!(matches!(
            serde_json::from_str::<AuthorName>("[]").unwrap(),
            AuthorName::List(names) if names.is_empty(),
        ));

        let a = AuthorName::One(String::from("John"));
        assert!(a.is_author("John"));
        assert!(!a.is_author("Alice"));
        assert_eq!("\"John\"", serde_json::to_string(&a).unwrap());

        let a = AuthorName::List(vec![String::from("Alice"), String::from("Bob")]);
        assert!(a.is_author("Alice"));
        assert!(!a.is_author("John"));
        assert_eq!("[\"Alice\",\"Bob\"]", serde_json::to_string(&a).unwrap());

        let a = AuthorName::List(vec![String::from("Alice"), String::from("Bob")]);
        assert!(a.is_author("Alice"));
        assert!(!a.is_author("John"));
        assert_eq!("[\"Alice\",\"Bob\"]", serde_json::to_string(&a).unwrap());
    }
}
