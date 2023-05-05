use std::{borrow::Cow, env, fs, io::Write, path::PathBuf};

use anyhow::{Context as _, Ok, Result};
use clap::{Arg, ArgAction, Command};
use genkit::{helpers, Cmd};
use minijinja::render;
use promptly::prompt_default;
use time::OffsetDateTime;

use crate::{entity::Zine, ZINE_FILE};

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

static TEMPLATE_ARTICLE: &str = r#"

[[article]]
file = "{{ file }}"
title = "{{ title }}"
author = "{{ author | lower }}"
cover = ""
pub_date = "{{ pub_date }}"
publish = true
featured = true
"#;

pub struct NewCmd;

#[async_trait::async_trait]
impl Cmd for NewCmd {
    fn on_init(&self) -> clap::Command {
        Command::new("new")
            .args([
                Arg::new("name").help("Name of the project").required(false),
                Arg::new("issue")
                    .long("issue")
                    .short('i')
                    .action(ArgAction::SetTrue)
                    .help("New issue."),
                Arg::new("article")
                    .long("article")
                    .short('a')
                    .action(ArgAction::SetTrue)
                    .conflicts_with("issue")
                    .help("New article."),
            ])
            .about("New a Zine project, issue or article")
    }

    async fn on_execute(&self, arg_matches: &clap::ArgMatches) -> anyhow::Result<()> {
        let issue = arg_matches.get_flag("issue");
        let article = arg_matches.get_flag("article");
        if issue {
            new_zine_issue()?;
        } else if article {
            new_article()?;
        } else {
            new_zine_project(arg_matches.get_one("name").cloned())?
        }

        Ok(())
    }
}

struct ZineScaffold {
    source: PathBuf,
    author: String,
    issue_dir: Cow<'static, str>,
    issue_number: usize,
    issue_title: Cow<'static, str>,
}

impl ZineScaffold {
    fn create_project(&self, name: &str) -> Result<()> {
        // Generate project zine.toml
        fs::write(
            self.source.join(ZINE_FILE),
            render!(TEMPLATE_PROJECT_FILE, name, author => self.author),
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

        fs::write(
            issue_dir.join(ZINE_FILE),
            render!(
                TEMPLATE_ISSUE_FILE,
                slug => self.issue_dir,
                number => self.issue_number,
                title => self.issue_title,
                pub_date => helpers::format_date(&OffsetDateTime::now_utc().date()),
                author => self.author
            ),
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

    let author = git_user_name();
    let scaffold = ZineScaffold {
        source,
        author,
        issue_dir: "issue-1".into(),
        issue_number: 1,
        issue_title: "Issue 1".into(),
    };

    scaffold.create_project(&name.unwrap_or_default())?;
    println!(
        r#"
    Created sucessfully!
    
    To start your magazine, run:    
    $ zine serve --open

    Or to build your magazine, run:
    $ zine build
    "#
    );
    Ok(())
}

fn load_zine_project() -> Result<(PathBuf, Zine)> {
    // Use zine.toml to find root path
    let (source, mut zine) = crate::locate_root_zine_folder(env::current_dir()?)?
        .with_context(|| "Failed to find the root zine.toml file".to_string())?;
    zine.parse_issue_from_dir(&source)?;
    Ok((source, zine))
}

pub fn new_zine_issue() -> Result<()> {
    let (source, zine) = load_zine_project()?;
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

    let author = git_user_name();
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

pub fn new_article() -> Result<()> {
    let (source, zine) = load_zine_project()?;
    let latest_issue_number = zine.issues.len();
    let issue_number = prompt_default(
        "Which Issue do you want create a new article?",
        latest_issue_number,
    )?;
    if let Some(issue) = zine.get_issue_by_number(issue_number as u32) {
        let article_file = prompt_default(
            "What is your article file name?",
            "new-article.md".to_owned(),
        )?;
        let title = prompt_default("What is your article title?", "New Article".to_owned())?;
        let author = git_user_name();

        let issue_dir = source.join(crate::ZINE_CONTENT_DIR).join(&issue.dir);
        // Write article file
        fs::write(issue_dir.join(&article_file), "Hello Zine")?;

        // Append article to issue zine.toml
        let article_content = render!(
            TEMPLATE_ARTICLE,
            title,
            author,
            file => article_file,
            pub_date => helpers::format_date(&OffsetDateTime::now_utc().date()),
        );
        let mut issue_file = fs::OpenOptions::new()
            .append(true)
            .open(issue_dir.join(ZINE_FILE))?;
        issue_file.write_all(article_content.as_bytes())?;
    } else {
        println!("Issue {} not found", issue_number);
    }

    Ok(())
}

fn git_user_name() -> String {
    helpers::run_command("git", &["config", "user.name"])
        .ok()
        .unwrap_or_default()
        .replace(' ', "_")
}
