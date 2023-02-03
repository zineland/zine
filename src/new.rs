use std::{borrow::Cow, env, fs, path::PathBuf};

use anyhow::{Context as _, Result};
use promptly::prompt_default;
use tera::{Context, Tera};
use time::{format_description, OffsetDateTime};

use crate::{helpers::run_command, ZINE_FILE};
use crate::{AuthorId, Author, Article, Issue, Site};

static TEMPLATE_PROJECT_FILE: &str = r#"
[site]
url = "http://localhost"
name = "{{ name }}"
description = ""

[authors]
{% if author -%}
{{ author | lower }} = { name = "{{ author }}" }
{% endif -%}
"#;

static TEMPLATE_ISSUE_FILE: &str = r#"
slug = "{{ slug }}"
number = {{ number }}
title = "{{ title }}"

[[article]]
file = "1-first.md"
title = "First article"
author = "{{ author | lower }}"
cover = ""
pub_date = "{{ pub_date }}"
publish = true
featured = true
"#;

struct ZineScaffold {
    source: PathBuf,
    author: String,
    issue_dir: Cow<'static, str>,
    issue_number: usize,
    issue_title: Cow<'static, str>,
}

impl ZineScaffold {
    fn create_project(&self, name: &str) -> Result<()> {
        let mut context = Context::new();
        context.insert("name", name);
        context.insert("author", &self.author);

        // Generate project zine.toml
        fs::write(
            self.source.join(ZINE_FILE),
            Tera::one_off(TEMPLATE_PROJECT_FILE, &context, true)?,
        )?;

        // Create issue dir and issue zine.toml
        self.create_issue()?;
        Ok(())
    }

    // Create issue dir and issue zine.toml
    fn create_issue(&self) -> Result<()> {
        let issue_dir = self
            .source
            .join(crate::ZINE_CONTENT_DIR)
            .join(self.issue_dir.as_ref());
        fs::create_dir_all(&issue_dir)?;
        let format = format_description::parse("[year]-[month]-[day]")?;
        let today = OffsetDateTime::now_utc().format(&format)?;

        let mut context = Context::new();
        context.insert("slug", &self.issue_dir);
        context.insert("number", &self.issue_number);
        context.insert("title", &self.issue_title);
        context.insert("pub_date", &today);
        context.insert("author", &self.author);

        fs::write(
            issue_dir.join(ZINE_FILE),
            Tera::one_off(TEMPLATE_ISSUE_FILE, &context, true)?,
        )?;

        // Create first article
        fs::write(issue_dir.join("1-first.md"), "Hello Zine")?;
        Ok(())
    }
}

pub fn new_zine_project(name: Option<String>) -> Result<()> {
    let source = if let Some(name) = name.as_ref() {
        env::current_dir()?.join(name)
    } else {
        env::current_dir()?
    };
    if !source.exists() {
        fs::create_dir_all(&source)?;
    }

    let author = run_command("git", &["config", "user.name"])
        .ok()
        .unwrap_or_default();
    let scaffold = ZineScaffold {
        source,
        author,
        issue_dir: "issue-1".into(),
        issue_number: 1,
        issue_title: "Issue 1".into(),
    };

    scaffold.create_project(&name.unwrap_or_default())?;
    Ok(())
}

pub fn new_zine_issue() -> Result<()> {
    // Use zine.toml to find root path
    let (source, mut zine) = crate::locate_root_zine_folder(env::current_dir()?)?
        .with_context(|| "Failed to find the root zine.toml file".to_string())?;
    zine.parse_issue_from_dir(&source)?;

    let author = run_command("git", &["config", "user.name"])
        .ok()
        .unwrap_or_default();
    let next_issue_number = zine.issues.len() + 1;
    let issue_dir = prompt_default(
        "What is your issue directory name?",
        format!("issue-{next_issue_number}"),
    )?;
    let issue_number = prompt_default("What is your issue number?", next_issue_number)?;
    let issue_title = prompt_default(
        "What is your issue title?",
        format!("Issue {next_issue_number}"),
    )?;

    let scaffold = ZineScaffold {
        source,
        author,
        issue_dir: issue_dir.into(),
        issue_number,
        issue_title: issue_title.into(),
    };
    scaffold.create_issue()?;
    Ok(())
}

#[derive(Default)]
pub struct SiteBuilder {
    source: PathBuf,
    site: Site,
}

impl SiteBuilder {
    // Defines a new site with default settings while providing a new an optional site `name`
    pub fn new(name: Option<String>) -> Result<Self> {
        let source = if let Some(name) = name.as_ref() {
            env::current_dir()?.join(name)
        } else {
            env::current_dir()?
        };
        Ok(Self {
            source,
            site: Site {
                name: name.unwrap_or_default(),
                ..Site::default()
            },
            ..Default::default()
        })
    }
    pub fn create_new_zine_magazine(&mut self) -> Result<()> {
        // Create Root of Zine Magazine
        if !self.source.exists() {
            std::fs::create_dir_all(&self.source)?;
        }
        if !self.source.join(crate::ZINE_FILE).exists() {
            // This requires that we add the author struct to Site
            /*
            let author = run_command("git", &["config", "user.name"])
                .ok()
                .unwrap_or_default();
            let author_id = author.parse::<AuthorId>()?;
            let author = Author {
                id: serde_json::to_string(&author_id).unwrap().to_lowercase(),
                ..Default::default()
            };
            self.site.authors.push(author);
            */
            let content_dir = &self.source.join(crate::ZINE_CONTENT_DIR);
            self.site.write_toml(&self.source.join(crate::ZINE_FILE))?;
            std::fs::create_dir_all(&content_dir)?;

            let mut issue = Issue::new();
            issue.set_title("issue").set_issue_number(1);
            issue = issue.finalize();
            issue.create_issue_dir(&content_dir)?;

            let article = Article::default();
            issue.articles.push(article);

            let issue_dir = &content_dir.join(&issue.dir);
            let toml_file = &issue_dir.join(crate::ZINE_FILE);
            // Write Issue zine.toml
            issue.write_new_issue(&content_dir)?;

            for article in &issue.articles {
                article.append_article_to_toml(&toml_file)?;
            }

            if !issue.articles.is_empty() {
                issue.write_initial_markdown_file(&content_dir)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod site_builder {

    use super::Site;
    use super::SiteBuilder;
    use tempfile::tempdir;

    #[test]
    fn site_to_build() {
        let temp_dir = tempdir().unwrap();
        let site = SiteBuilder::default();
        assert_eq!(site.site.url, "http://localhost");

        let file_path = temp_dir.path().join("dummy.toml");
        assert!(site.site.write_toml(&file_path.as_path()).is_ok());

        let read_contents = std::fs::read_to_string(&file_path).unwrap();
        let data: Site = toml::from_str(&read_contents).unwrap();

        assert_eq!(data.name, "My New Magazine Powered by Rust!");
        assert_eq!(data.url, "http://localhost");

        drop(file_path);
        assert!(temp_dir.close().is_ok());
    }

    #[test]
    fn test_site_builder_new() {
        let new_site = SiteBuilder::new(Some("test".to_string()));
        let temp_dir = tempdir().unwrap();

        if let Ok(new_site) = new_site {
            let file_path = temp_dir.path().join("dummy.toml");
            assert_eq!(new_site.site.name, "test".to_string());
            assert!(new_site.site.write_toml(&file_path).is_ok());

            let read_contents = std::fs::read_to_string(&file_path).unwrap();
            let data: Site = toml::from_str(&read_contents).unwrap();

            assert_eq!(data.name, "test");
            assert_eq!(data.url, "http://localhost");

            drop(file_path);
            assert!(temp_dir.close().is_ok());
        }
    }
}
