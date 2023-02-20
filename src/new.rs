use std::io::prelude::*;
use std::{borrow::Cow, env, fs, path::PathBuf};

use anyhow::{Context as _, Result};
use promptly::prompt_default;

use crate::{helpers::get_author_from_git, helpers::run_command, ZINE_CONTENT_DIR, ZINE_FILE};
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
            name: Some(self.author.clone()),
            id: self.author.to_lowercase(),
            ..Default::default()
        };
        site.write_toml(&self.source.join(ZINE_FILE))?;

        // Appending Author to site zine.toml
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(self.source.join(crate::ZINE_FILE))?;
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
        fs::create_dir_all(contents_dir.join(&issue.dir))?;
        let mut article = Article::default();
        article.set_authors(&self.author)?.finalize();
        issue.add_article(article);
        issue.write_new_issue(&contents_dir)?;

        if !issue.articles.is_empty() {
            issue
                .articles
                .first()
                .expect("No Articles in issue")
                .write_markdown_template(&contents_dir.join(&issue.dir))?;
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

    let author = git_user_name();
    let scaffold = ZineScaffold {
        source,
        author,
        issue_number: 1,
        issue_title: "Issue".into(),
    };

    scaffold.create_project(&name.unwrap_or_default())?;
    Ok(())
}

pub fn new_zine_issue() -> Result<()> {
    // Use zine.toml to find root path
    let (source, mut zine) = crate::locate_root_zine_folder(env::current_dir()?)?
        .with_context(|| "Failed to find the root zine.toml file".to_string())?;
    zine.parse_issue_from_dir(&source)?;

    let author = git_user_name();
    let next_issue_number = zine.issues.len() + 1;

    let issue_number = prompt_default("What is your issue number?", next_issue_number)?;
    let issue_title = prompt_default("What is your issue title?", "Issue".to_string())?;

    let scaffold = ZineScaffold {
        source,
        author,
        issue_number: issue_number.try_into().unwrap(),
        issue_title: issue_title.into(),
    };
    scaffold.create_issue()?;
    Ok(())
}

pub fn new_zine_article() -> Result<()> {
    // Use zine.toml to find root path
    let (source, mut zine) = crate::locate_root_zine_folder(env::current_dir()?)?
        .with_context(|| "Failed to find the root zine.toml file".to_string())?;
    zine.parse_issue_from_dir(&source)?;

    let author = get_author_from_git().to_lowercase();

    let article_title = prompt_default(
        "What is your article's title?",
        "My New Article".to_string(),
    )?;
    let author = prompt_default("Author or list of author names (lowercase):", author)?;

    let article = Article::default()
        .set_title(article_title.as_ref())
        .set_authors(author.as_ref())?
        .finalize();

    let issue_path = &source
        .join(ZINE_CONTENT_DIR)
        .join(&zine.issues.first().expect("No Issues found.").dir);

    // Append the new article to the zine.toml for the current Issue.
    article.append_article_to_toml(&issue_path.join(ZINE_FILE))?;
    // vec.push() places the new article at the start of the vec. So it is at index:0
    zine.issues[0].add_article(article);
    zine.issues[0]
        .articles
        .last()
        .expect("No Articles for this Issue")
        .write_markdown_template(issue_path)?;
    Ok(())
}
fn git_user_name() -> String {
    run_command("git", &["config", "user.name"])
        .ok()
        .unwrap_or_default()
        .replace(' ', "_")
}
