use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use anyhow::Result;
use minijinja::Environment;
use serde::{Deserialize, Serialize};

use crate::engine;
use genkit::{html::Meta, markdown, Context};

use super::Entity;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Page {
    // The page's markdown content.
    pub markdown: String,
    // Relative path of page file.
    pub file_path: PathBuf,
}

impl Page {
    pub fn slug(&self) -> String {
        self.file_path.to_str().unwrap().replace(".md", "")
    }

    fn title(&self) -> String {
        let prefix = &['#', ' '];
        self.markdown
            .lines()
            .find_map(|line| {
                if line.starts_with(prefix) {
                    Some(line.trim_start_matches(prefix).to_owned())
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }
}

impl Entity for Page {
    fn render(&self, env: &Environment, mut context: Context, dest: &Path) -> Result<()> {
        context.insert(
            "meta",
            &Meta {
                title: Cow::Borrowed(&self.title()),
                description: Cow::Owned(markdown::extract_description(&self.markdown)),
                url: Some(Cow::Owned(self.slug())),
                image: None,
            },
        );
        context.insert("page", &self);
        engine::render(env, "page.jinja", context, dest.join(self.slug()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use test_case::test_case;

    use super::Page;

    #[test_case("  # Title
    aaa"; "case0")]
    #[test_case("# Title
    aaa"; "case1")]
    #[test_case("## Title
    aaa"; "case2")]
    #[test_case("

    # Title
    aaa"; "case3")]
    #[test_case("
    # Title
    aaa"; "case4")]
    #[test_case("
    # Title
    ## Subtitle
    aaa"; "case5")]
    fn test_parse_page_title(markdown: &str) {
        let page = Page {
            markdown: markdown.to_owned(),
            file_path: PathBuf::new(),
        };

        assert_eq!("Title", page.title());
    }
}
