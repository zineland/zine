use std::io::prelude::*;
use std::{borrow::Cow, env, fs, path::PathBuf};

use anyhow::{Context as _, Result};
use promptly::prompt_default;

use crate::{helpers::run_command, ZINE_FILE};
use crate::{Article, Author, Issue, Site};

struct ZineScaffold {
    source: PathBuf,
    author: String,
    issue_number: u32,
    issue_title: Cow<'static, str>,
}

impl ZineScaffold {
    fn create_project(&self, name: &str) -> Result<()> {
        let site = Site {
            name: name.into(),
            ..Default::default()
        };
        let author = Author {
            name: Some(self.author.clone().to_lowercase()),
            id: self.author.to_lowercase(),
            ..Default::default()
        };
        site.write_toml(&self.source.join(ZINE_FILE))?;

        // Appending Author to site zine.toml
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&self.source.join(crate::ZINE_FILE))?;
        file.write_all("\n[authors]\n".as_bytes())?;
        file.write_all(author.to_string().as_bytes())?;

        // Create issue dir and issue zine.toml
        self.create_issue()?;
        Ok(())
    }

    // Create issue dir and issue zine.toml
    fn create_issue(&self) -> Result<()> {
        let contents_dir = self.source.join(crate::ZINE_CONTENT_DIR);

        let mut issue = Issue::new()
            .set_title(self.issue_title.clone())
            .set_issue_number(self.issue_number)
            .finalize();
        println!("{:?}", issue);
        fs::create_dir_all(&contents_dir.join(&issue.dir))?;
        let mut article = Article::default();
        article.meta.set_authors(&self.author)?.finalize();
        article.finalize();
        issue.add_article(article);
        issue.write_new_issue(&contents_dir)?;

        if !issue.articles.is_empty() {
            issue.write_initial_markdown_file(&contents_dir)?;
            issue.articles[0]
                .append_article_to_toml(&contents_dir.join(issue.dir).join(ZINE_FILE))?;
        }

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

    let issue_number = prompt_default("What is your issue number?", next_issue_number)?;
    let issue_title = prompt_default("What is your issue title?", format!("Issue"))?;

    let scaffold = ZineScaffold {
        source,
        author,
        issue_number: issue_number.try_into().unwrap(),
        issue_title: issue_title.into(),
    };
    scaffold.create_issue()?;
    Ok(())
}
