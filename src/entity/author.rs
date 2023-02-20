use std::{borrow::Cow, path::Path};

use anyhow::Result;
use serde::{de, ser::SerializeSeq, Deserialize, Serialize};
use tera::Context;

use crate::{engine, error::ZineError, html::Meta, markdown, Entity};

/// AuthorId represents a single author or multiple co-authors.
/// Declared in `[[article]]` table.
#[derive(Debug, Clone)]
pub enum AuthorId {
    // Single author.
    One(String),
    // Co-authors.
    List(Vec<String>),
}
/// Provides a parser to create AuthorId Structs
/// Strings should be in the form of a space delimited list of names
impl std::str::FromStr for AuthorId {
    type Err = ZineError;
    /// Creates a AuthorId Struct from string imput. The string should be `space` delimited
    /// Note: Addtional checks should be added for sanity
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut author_id_vec = vec![];
        if s.contains(' ') {
            let s_inter = s.split_whitespace();

            for author_id in s_inter {
                // Removed character checking. This needs to be reconsidered
                author_id_vec.push(author_id.into());
            }
            Ok::<AuthorId, ZineError>(AuthorId::List(author_id_vec))
        } else {
            Ok::<AuthorId, ZineError>(AuthorId::One(s.into()))
        }
    }
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

/// The author of an article. Declared in the root `zine.toml`'s **\[authors\]** table.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
/// Implementation for Display. Will return a JSON style string for writing to the `Site` TOML file.
impl std::fmt::Display for Author {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} = {{ name = \"{}\", editor = {}, bio = \"\"\"\n{}\n\"\"\" }}",
            self.id,
            match &self.name {
                Some(name) => name,
                None => "",
            },
            match self.editor {
                true => "true",
                false => "false",
            },
            // This should probably stay at the end of the file for now
            match &self.bio {
                Some(data) => data,
                None => "",
            },
        )
    }
}

#[cfg(test)]
mod author_tests {
    use crate::entity::Author;

    #[test]
    fn author() {
        let author = Author {
            id: "bob".into(),
            name: Some("Bob".into()),
            avatar: None,
            bio: None,
            editor: false,
            team: false,
        };
        println!("{}", author.to_string())
    }
}
impl Entity for Author {
    fn render(&self, mut context: Context, dest: &Path) -> anyhow::Result<()> {
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
        engine::render("author.jinja", &context, dest.join(slug))?;
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

    #[test]
    fn author_id_parser() {
        assert!(matches!(
                "Alice".parse().unwrap(),
                AuthorId::One(name) if name == *"Alice",
        ));
        assert!(matches!(
                "Alice Bob".parse().unwrap(),
                AuthorId::List(names) if names == vec![String::from("Alice"), String::from("Bob")],
        ));
        let a: AuthorId = "Alice Bob".parse().unwrap();
        assert!(a.is_author("Alice"));
        assert!(a.is_author("Bob"));
        assert!(!a.is_author("John"));
    }
}
