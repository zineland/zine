use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    context::Context,
    current_mode, data,
    entity::{Entity, MarkdownConfig, Zine},
    helpers::copy_dir,
    html::rewrite_html_base_url,
    locales::FluentLoader,
    markdown::MarkdownRender,
    Mode,
};

use anyhow::Result;
use hyper::Uri;
use minijinja::{context, value::Value as JinjaValue, Environment};
use serde::Serialize;
use serde_json::Value;

// The `Environment` only includes the fundamental functions and filters.
// It is used for rendering html in some code blocks, such as `QuoteBlock`.
pub fn init_lite_jinja_environment<'a>() -> Environment<'a> {
    let mut env = Environment::new();
    env.add_function("markdown_to_html", markdown_to_html_function);
    env.add_function("get_author", get_author_function);

    let templates = [
        ("heading.jinja", include_str!("../templates/heading.jinja")),
        (
            "blocks/quote.jinja",
            include_str!("../templates/blocks/quote.jinja"),
        ),
    ];
    for (name, template) in templates {
        env.add_template(name, template).unwrap();
    }
    env
}

// The `Environment` includes all the templates and functions.
fn init_jinja_environment<'a>(source: &Path, zine: &'a Zine) -> Environment<'a> {
    let mut env = init_lite_jinja_environment();
    #[cfg(debug_assertions)]
    env.set_source(minijinja::Source::from_path("templates"));

    env.add_global("site", JinjaValue::from_serializable(&zine.site));
    env.add_global("theme", JinjaValue::from_serializable(&zine.theme));
    env.add_global(
        "zine_version",
        option_env!("CARGO_PKG_VERSION").unwrap_or("(Unknown Cargo package version)"),
    );
    env.add_global(
        "live_reload",
        matches!(crate::current_mode(), crate::Mode::Serve),
    );

    #[cfg(not(debug_assertions))]
    {
        let templates = [
            (
                "_article_ref.jinja",
                include_str!("../templates/_article_ref.jinja"),
            ),
            ("_macros.jinja", include_str!("../templates/_macros.jinja")),
            ("_meta.jinja", include_str!("../templates/_meta.jinja")),
            ("base.jinja", include_str!("../templates/base.jinja")),
            ("index.jinja", include_str!("../templates/index.jinja")),
            ("issue.jinja", include_str!("../templates/issue.jinja")),
            ("article.jinja", include_str!("../templates/article.jinja")),
            ("author.jinja", include_str!("../templates/author.jinja")),
            (
                "author-list.jinja",
                include_str!("../templates/author-list.jinja"),
            ),
            ("topic.jinja", include_str!("../templates/topic.jinja")),
            (
                "topic-list.jinja",
                include_str!("../templates/topic-list.jinja"),
            ),
            ("page.jinja", include_str!("../templates/page.jinja")),
            ("feed.jinja", include_str!("../templates/feed.jinja")),
            ("sitemap.jinja", include_str!("../templates/sitemap.jinja")),
        ];
        for (name, template) in templates {
            env.add_template(name, template).unwrap();
        }
    }

    // Dynamically add templates.
    if let Some(head_template) = &zine.theme.head_template {
        env.add_template("head_template.jinja", head_template)
            .expect("Cannot add head_template");
    }
    if let Some(footer_template) = &zine.theme.footer_template {
        env.add_template("footer_template.jinja", footer_template)
            .expect("Cannot add footer_template");
    }
    if let Some(article_extend_template) = &zine.theme.article_extend_template {
        env.add_template("article_extend_template.jinja", article_extend_template)
            .expect("Cannot add article_extend_template");
    }
    env.add_function("now", now_function);
    env.add_filter("trim_start_matches", trim_start_matches_filter);
    env.add_function("markdown_to_rss", markdown_to_rss_function);

    let fluent_loader = FluentLoader::new(source, &zine.site.locale);
    env.add_function("fluent", move |key: &str, number: Option<i64>| -> String {
        fluent_loader.format(key, number)
    });
    env
}

#[derive(Debug)]
pub struct ZineEngine {
    pub source: PathBuf,
    pub dest: PathBuf,
    zine: Zine,
}

pub fn render(
    env: &Environment,
    template: &str,
    context: Context,
    dest: impl AsRef<Path>,
) -> Result<()> {
    let mut buf = vec![];
    let dest = dest.as_ref().join("index.html");
    if let Some(parent_dir) = dest.parent() {
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir)?;
        }
    }

    let site = context.get("site").cloned();
    env.get_template(template)?
        .render_to_write(context.into_json(), &mut buf)?;

    // Rewrite some site url and cdn links if and only if:
    // 1. in build run mode
    // 2. site url has a path
    if matches!(current_mode(), Mode::Build) {
        let mut site_url: Option<&str> = None;
        let mut cdn_url: Option<&str> = None;

        if let Some(Value::String(url)) = site.as_ref().and_then(|site| site.get("cdn")) {
            let _ = url.parse::<Uri>().expect("Invalid cdn url.");
            cdn_url = Some(url);
        }
        if let Some(Value::String(url)) = site.as_ref().and_then(|site| site.get("url")) {
            let uri = url.parse::<Uri>().expect("Invalid site url.");
            // We don't need to rewrite links if the site url has a root path.
            if uri.path() != "/" {
                site_url = Some(url);
            }
        }

        let html = rewrite_html_base_url(&buf, site_url, cdn_url)?;
        fs::write(dest, html)?;
        return Ok(());
    }

    fs::write(dest, buf)?;
    Ok(())
}

// Render Atom feed
fn render_atom_feed(
    env: &Environment,
    context: impl Serialize,
    dest: impl AsRef<Path>,
) -> Result<()> {
    let dest = dest.as_ref().join("feed.xml");
    let template = env.get_template("feed.jinja")?;

    // tokio::task::spawn_blocking(move || {
    let mut buf = vec![];

    template
        .render_to_write(context, &mut buf)
        .expect("Render feed.jinja failed.");
    fs::write(dest, buf).expect("Write feed.xml failed");
    Ok(())
}

// Render sitemap.xml
fn render_sitemap(
    env: &Environment,
    context: impl Serialize,
    dest: impl AsRef<Path>,
) -> Result<()> {
    let dest = dest.as_ref().join("sitemap.xml");
    let template = env.get_template("sitemap.jinja")?;
    let mut buf = vec![];
    template
        .render_to_write(context, &mut buf)
        .expect("Render sitemap.jinja failed.");
    fs::write(dest, buf).expect("Write sitemap.xml failed");
    Ok(())
}

impl ZineEngine {
    pub fn new(source: impl AsRef<Path>, dest: impl AsRef<Path>, zine: Zine) -> Result<Self> {
        let dest = dest.as_ref().to_path_buf();
        if !dest.exists() {
            fs::create_dir_all(&dest)?;
        }
        Ok(ZineEngine {
            source: source.as_ref().to_path_buf(),
            dest,
            zine,
        })
    }

    fn copy_static_assets(&self) -> Result<()> {
        let static_dir = self.source.join("static");
        if static_dir.exists() {
            copy_dir(&static_dir, &self.dest)?;
        }

        // Copy builtin static files into dest static dir.
        let dest_static_dir = self.dest.join("static");
        #[allow(clippy::needless_borrow)]
        fs::create_dir_all(&dest_static_dir)?;

        #[cfg(not(debug_assertions))]
        include_dir::include_dir!("static").extract(dest_static_dir)?;
        // Alwasy copy static directory in debug mode.
        #[cfg(debug_assertions)]
        copy_dir(Path::new("./static"), &self.dest)?;

        Ok(())
    }

    pub fn build(&mut self, reload: bool) -> Result<()> {
        let instant = std::time::Instant::now();

        if reload {
            self.zine = Zine::parse_from_toml(&self.source)?;
        }

        self.zine.parse(&self.source)?;

        let env = init_jinja_environment(&self.source, &self.zine);

        self.zine
            .render(&env, Context::new(), &self.dest)
            .expect("Render zine failed.");
        #[cfg(debug_assertions)]
        println!("Zine engine: {:?}", self.zine);

        render_atom_feed(
            &env,
            context! {
                site => &self.zine.site,
                entries => &self.zine.latest_feed_entries(20),
                generator_version => env!("CARGO_PKG_VERSION"),
            },
            &self.dest,
        )?;

        render_sitemap(
            &env,
            context! {
                site => &self.zine.site,
                entries => &self.zine.sitemap_entries(),
            },
            &self.dest,
        )?;

        self.copy_static_assets()?;
        println!("Build cost: {}ms", instant.elapsed().as_millis());
        Ok(())
    }
}

fn now_function() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .expect("Failed to format now time.")
}

fn trim_start_matches_filter(s: &str, prefix: &str) -> String {
    s.trim_start_matches(prefix).to_string()
}

fn markdown_to_html_function(markdown: &str) -> String {
    let zine_data = data::read();
    let markdown_config = zine_data.get_markdown_config();
    MarkdownRender::new(markdown_config).render_html(markdown)
}

fn markdown_to_rss_function(markdown: &str) -> String {
    let markdown_config = MarkdownConfig {
        highlight_code: false,
        ..Default::default()
    };
    MarkdownRender::new(&markdown_config)
        .enable_rss_mode()
        .render_html(markdown)
}

fn get_author_function(id: &str) -> JinjaValue {
    let data = data::read();
    let author = data.get_author_by_id(id);
    JinjaValue::from_serializable(&author)
}
