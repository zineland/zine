use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    current_mode, data,
    entity::{Entity, Zine},
    helpers::copy_dir,
    html::rewrite_html_base_url,
    locales::FluentLoader,
    markdown::MarkdownRender,
    Mode,
};

use anyhow::Result;
use hyper::Uri;
use once_cell::sync::OnceCell;
use serde_json::Value;
use tera::{Context, Tera};

static TERA: OnceCell<parking_lot::RwLock<Tera>> = OnceCell::new();

fn init_tera(source: &Path, zine: &Zine) {
    TERA.get_or_init(|| {
        // Debug version tera which need to reload templates.
        #[cfg(debug_assertions)]
        let mut tera = Tera::new("templates/**/*.jinja").expect("Invalid template dir.");

        // Release version tera which not need to reload templates.
        #[cfg(not(debug_assertions))]
        let mut tera = Tera::default();
        #[cfg(not(debug_assertions))]
        tera.add_raw_templates(vec![
            (
                "_article_ref.jinja",
                include_str!("../templates/_article_ref.jinja"),
            ),
            ("_macros.jinja", include_str!("../templates/_macros.jinja")),
            ("_meta.jinja", include_str!("../templates/_meta.jinja")),
            ("heading.jinja", include_str!("../templates/heading.jinja")),
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
            (
                "blocks/quote.jinja",
                include_str!("../templates/blocks/quote.jinja"),
            ),
        ])
        .unwrap();
        tera.register_function("markdown_to_html", markdown_to_html_fn);
        tera.register_function("get_author", get_author_fn);

        parking_lot::RwLock::new(tera)
    });

    let mut tera = TERA.get().expect("Tera haven't initialized").write();

    // Full realod tera templates in debug mode.
    // Notice: the full reloading should take place before adding dynamic templates.
    #[cfg(debug_assertions)]
    tera.full_reload().expect("reload tera template failed");

    // Dynamically add templates.
    if let Some(head_template) = zine.theme.head_template.as_ref() {
        tera.add_raw_template("head_template.jinja", head_template)
            .expect("Cannot add head_template");
    }
    if let Some(footer_template) = zine.theme.footer_template.as_ref() {
        tera.add_raw_template("footer_template.jinja", footer_template)
            .expect("Cannot add footer_template");
    }
    if let Some(article_extend_template) = zine.theme.article_extend_template.as_ref() {
        tera.add_raw_template("article_extend_template.jinja", article_extend_template)
            .expect("Cannot add article_extend_template");
    }

    // Dynamically register functions that need dynamic configuration.
    tera.register_function("fluent", FluentLoader::new(source, &zine.site.locale));
}

/// Get a Tera under read lock.
pub fn get_tera() -> parking_lot::RwLockReadGuard<'static, Tera> {
    TERA.get().expect("Tera haven't initialized").read()
}

#[derive(Debug)]
pub struct ZineEngine {
    pub source: PathBuf,
    pub dest: PathBuf,
    zine: Zine,
}

pub fn render(template: &str, context: &Context, dest: impl AsRef<Path>) -> Result<()> {
    let mut buf = vec![];
    let dest = dest.as_ref().join("index.html");
    if let Some(parent_dir) = dest.parent() {
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir)?;
        }
    }

    get_tera().render_to(template, context, &mut buf)?;

    // Rewrite root path links with site url if and only if:
    // 1. in build run mode
    // 2. site url has a path
    if matches!(current_mode(), Mode::Build) {
        let mut site_url: Option<&str> = None;
        let mut cdn_url: Option<&str> = None;

        if let Some(Value::String(url)) = context.get("site").and_then(|site| site.get("cdn")) {
            let _ = url.parse::<Uri>().expect("Invalid cdn url.");
            cdn_url = Some(url);
        }
        if let Some(Value::String(url)) = context.get("site").and_then(|site| site.get("url")) {
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

/// Render raw template.
pub fn render_str(raw_template: &str, context: &Context) -> Result<String> {
    let mut tera = TERA.get().expect("Tera haven't initialized").write();
    let r = tera.render_str(raw_template, context)?;
    Ok(r)
}

// Render Atom feed
fn render_atom_feed(context: Context, dest: impl AsRef<Path>) -> Result<()> {
    let dest = dest.as_ref().join("feed.xml");
    tokio::task::spawn_blocking(move || {
        let mut buf = vec![];
        get_tera()
            .render_to("feed.jinja", &context, &mut buf)
            .expect("Render feed.jinja failed.");
        fs::write(dest, buf).expect("Write feed.xml failed");
    });
    Ok(())
}

// Render sitemap.xml
fn render_sitemap(context: Context, dest: impl AsRef<Path>) -> Result<()> {
    let dest = dest.as_ref().join("sitemap.xml");
    tokio::task::spawn_blocking(move || {
        let mut buf = vec![];
        get_tera()
            .render_to("sitemap.jinja", &context, &mut buf)
            .expect("Render sitemap.jinja failed.");
        fs::write(dest, buf).expect("Write sitemap.xml failed");
    });
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
        fs::create_dir_all(&dest_static_dir)?;

        #[cfg(not(debug_assertions))]
        include_dir::include_dir!("static").extract(dest_static_dir)?;
        // Alwasy copy static directory in debug mode.
        #[cfg(debug_assertions)]
        copy_dir(Path::new("./static"), &self.dest)?;

        Ok(())
    }

    pub fn build(&mut self, reload: bool) -> Result<()> {
        if reload {
            self.zine = Zine::parse_from_toml(&self.source)?;
        }

        self.zine.parse(&self.source)?;

        init_tera(&self.source, &self.zine);

        self.zine.render(Context::new(), &self.dest)?;
        #[cfg(debug_assertions)]
        println!("Zine engine: {:?}", self.zine);

        let mut feed_context = Context::new();
        feed_context.insert("site", &self.zine.site);
        feed_context.insert("entries", &self.zine.latest_feed_entries(20));
        feed_context.insert("generator_version", env!("CARGO_PKG_VERSION"));
        render_atom_feed(feed_context, &self.dest)?;

        let mut sitemap_context = Context::new();
        sitemap_context.insert("site", &self.zine.site);
        sitemap_context.insert("entries", &self.zine.sitemap_entries());
        render_sitemap(sitemap_context, &self.dest)?;

        self.copy_static_assets()
    }
}

// A tera function to convert markdown into html.
fn markdown_to_html_fn(map: &HashMap<String, Value>) -> tera::Result<Value> {
    if let Some(Value::String(markdown)) = map.get("markdown") {
        let zine_data = data::read();
        let markdown_config = zine_data.get_markdown_config();
        let html = MarkdownRender::new(markdown_config).render_html(markdown);
        Ok(Value::String(html))
    } else {
        Ok(Value::Array(vec![]))
    }
}

fn get_author_fn(map: &HashMap<String, Value>) -> tera::Result<Value> {
    if let Some(Value::String(author_id)) = map.get("id") {
        let data = data::read();
        let author = data.get_author_by_id(author_id);
        Ok(serde_json::to_value(author)?)
    } else {
        Ok(Value::Null)
    }
}
