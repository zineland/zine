use std::{borrow::Cow, env, fs, path::PathBuf};

use anyhow::{Context as _, Result};
use promptly::prompt_default;
use tera::{Context, Tera};
use time::{format_description, OffsetDateTime};

use crate::ZINE_FILE;

static TEMPLATE_PROJECT_FILE: &str = r#"
[site]
url = "http://localhost"
name = "{{ name }}"
description = ""

[authors]
zine-team = { name = "Zine Team" }
"#;

static TEMPLATE_ISSUE_FILE: &str = r#"
slug = "{{ slug }}"
number = {{ number }}
title = "{{ title }}"

[[article]]
file = "1-first.md"
title = "First article"
author = "zine-team"
cover = ""
pub_date = "{{ pub_date }}"
publish = true
featured = true
"#;

struct ZineScaffold {
    source: PathBuf,
    issue_dir: Cow<'static, str>,
    number: usize,
    title: Cow<'static, str>,
}

impl ZineScaffold {
    fn create_project(&self, name: &str) -> Result<()> {
        let mut context = Context::new();
        context.insert("name", name);

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
        context.insert("number", &self.number);
        context.insert("title", &self.title);
        context.insert("pub_date", &today);

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

    let scaffold = ZineScaffold {
        source,
        issue_dir: "issue-1".into(),
        number: 1,
        title: "Issue 1".into(),
    };

    scaffold.create_project(&name.unwrap_or_default())?;
    Ok(())
}

pub fn new_zine_issue() -> Result<()> {
    // Use zine.toml to find root path
    let (source, mut zine) = crate::locate_root_zine_folder(env::current_dir()?)?
        .with_context(|| "Failed to find the root zine.toml file".to_string())?;
    zine.parse_issue_from_dir(&source)?;

    let next_issue_number = zine.issues.len() + 1;
    let issue_dir = prompt_default(
        "What is your issue directory name?",
        format!("issue{next_issue_number}"),
    )?;
    let number = prompt_default("What is your issue number?", next_issue_number)?;
    let title = prompt_default(
        "What is your issue title?",
        format!("Issue-{next_issue_number}"),
    )?;

    let scaffold = ZineScaffold {
        source,
        issue_dir: issue_dir.into(),
        number,
        title: title.into(),
    };
    scaffold.create_issue()?;
    Ok(())
}
