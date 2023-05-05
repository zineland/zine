use std::{borrow::Cow, path::Path};

use anyhow::Result;
use genkit::{html::Meta, markdown, Context, Entity};
use minijinja::Environment;
use serde::{de, ser::SerializeSeq, Deserialize, Serialize};

use crate::engine;

/// AuthorId represents a single author or multiple co-authors.
/// Declared in `[[article]]` table.
#[derive(Debug, Clone)]
pub enum AuthorId {
    // Single author.
    One(String),
    // Co-authors.
    List(Vec<String>),
}

/// The author of an article. Declared in the root `zine.toml`'s **[authors]** table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    /// The author id, which is the key declared in `[authors]` table.
    #[serde(skip_deserializing, default)]
    pub id: String,
    /// The author's name. Will fallback to capitalized id if missing.
    pub name: Option<String>,
    /// The optional avatar url. Will fallback to default zine logo if missing.
    pub avatar: Option<String>,
    /// The bio of author (markdown format).
    pub bio: Option<String>,
    /// Whether the author is an editor.
    #[serde(default)]
    pub editor: bool,
    #[serde(default)]
    /// Whether the author is a team account.
    pub team: bool,
}

impl AuthorId {
    pub fn is_author(&self, id: &str) -> bool {
        match self {
            Self::One(author_id) => author_id.eq_ignore_ascii_case(id),
            Self::List(authors) => authors
                .iter()
                .any(|author_id| author_id.eq_ignore_ascii_case(id)),
        }
    }
}

impl Entity for Author {
    fn render(&self, env: &Environment, mut context: Context, dest: &Path) -> anyhow::Result<()> {
        let slug = format!("@{}", self.id.to_lowercase());
        context.insert(
            "meta",
            &Meta {
                title: Cow::Borrowed(self.name.as_deref().unwrap_or(&self.id)),
                description: Cow::Owned(
                    self.bio
                        .as_ref()
                        .map(|bio| markdown::extract_description(bio))
                        .unwrap_or_default(),
                ),
                url: Some(Cow::Borrowed(&slug)),
                image: None,
            },
        );
        context.insert("author", &self);
        engine::render(env, "author.jinja", context, dest.join(slug))?;
        Ok(())
    }
}

impl<'de> Deserialize<'de> for AuthorId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(AuthorNameVisitor)
    }
}

impl Serialize for AuthorId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            AuthorId::One(author) => serializer.serialize_str(author),
            AuthorId::List(authors) => {
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
    type Value = AuthorId;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("plain string or string list")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(AuthorId::One(v.to_string()))
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
        Ok(AuthorId::List(authors))
    }
}

#[cfg(test)]
mod tests {
    use super::AuthorId;

    #[test]
    fn test_author_name() {
        assert!(matches!(
            serde_json::from_str::<AuthorId>("\"Alice\"").unwrap(),
            AuthorId::One(name) if name == *"Alice",
        ));
        assert!(matches!(
            serde_json::from_str::<AuthorId>("[\"Alice\",\"Bob\"]").unwrap(),
            AuthorId::List(names) if names == vec![String::from("Alice"), String::from("Bob")],
        ));
        assert!(matches!(
            serde_json::from_str::<AuthorId>("[\"Alice\",\"Bob\", \"Alice\"]").unwrap(),
            AuthorId::List(names) if names == vec![String::from("Alice"), String::from("Bob")],
        ));
        assert!(matches!(
            serde_json::from_str::<AuthorId>("[]").unwrap(),
            AuthorId::List(names) if names.is_empty(),
        ));

        let a = AuthorId::One(String::from("John"));
        assert!(a.is_author("John"));
        assert!(!a.is_author("Alice"));
        assert_eq!("\"John\"", serde_json::to_string(&a).unwrap());

        let a = AuthorId::List(vec![String::from("Alice"), String::from("Bob")]);
        assert!(a.is_author("Alice"));
        assert!(!a.is_author("John"));
        assert_eq!("[\"Alice\",\"Bob\"]", serde_json::to_string(&a).unwrap());

        let a = AuthorId::List(vec![String::from("Alice"), String::from("Bob")]);
        assert!(a.is_author("Alice"));
        assert!(!a.is_author("John"));
        assert_eq!("[\"Alice\",\"Bob\"]", serde_json::to_string(&a).unwrap());
    }
}
