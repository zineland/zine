use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    code_blocks::{render_code_block, AuthorCode, CodeBlock, ALL_CODE_BLOCKS},
    current_mode, data,
    entity::{Entity, Zine},
    html::rewrite_html_base_url,
    locales::FluentLoader,
    Mode,
};

use anyhow::{Context as _, Result};
use hyper::Uri;
use once_cell::sync::OnceCell;
use serde_json::Value;
use syntect::{highlighting::ThemeSet, html::highlighted_html_for_string, parsing::SyntaxSet};
use tera::{Context, Tera};
use tokio::{runtime::Handle, task};

#[cfg(not(debug_assertions))]
static TERA: OnceCell<std::sync::Arc<Tera>> = OnceCell::new();
#[cfg(debug_assertions)]
static TERA: OnceCell<parking_lot::RwLock<Tera>> = OnceCell::new();

fn init_tera(source: &Path, locale: &str) {
    TERA.get_or_init(|| {
        // Debug version tera which need to reload templates.
        #[cfg(debug_assertions)]
        let mut tera = Tera::new("templates/*.jinja").expect("Invalid template dir.");

        // Release version tera which not need to reload templates.
        #[cfg(not(debug_assertions))]
        let mut tera = Tera::default();
        #[cfg(not(debug_assertions))]
        tera.add_raw_templates(vec![
            ("_macros.jinja", include_str!("../templates/_macros.jinja")),
            (
                "_anchor-link.jinja",
                include_str!("../templates/_anchor-link.jinja"),
            ),
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
            ("page.jinja", include_str!("../templates/page.jinja")),
            ("feed.jinja", include_str!("../templates/feed.jinja")),
            ("sitemap.jinja", include_str!("../templates/sitemap.jinja")),
        ])
        .unwrap();
        tera.register_function("markdown_to_html", markdown_to_html_fn);
        tera.register_function("get_author", get_author_fn);
        tera.register_function("fluent", FluentLoader::new(source, locale));

        #[cfg(debug_assertions)]
        return parking_lot::RwLock::new(tera);
        #[cfg(not(debug_assertions))]
        return std::sync::Arc::new(tera);
    });
    #[cfg(debug_assertions)]
    {
        // Full realod tera templates.
        TERA.get()
            .expect("Tera haven't initialized")
            .write()
            .full_reload()
            .expect("reload tera template failed");
    }
}

#[cfg(not(debug_assertions))]
fn get_tera() -> &'static std::sync::Arc<Tera> {
    TERA.get().expect("Tera haven't initialized")
}

#[cfg(debug_assertions)]
fn get_tera() -> parking_lot::RwLockReadGuard<'static, Tera> {
    TERA.get().expect("Tera haven't initialized").read()
}

#[derive(Debug)]
pub struct ZineEngine {
    source: PathBuf,
    dest: PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub struct Render;

impl Render {
    pub fn render(template: &str, context: &Context, dest: impl AsRef<Path>) -> Result<()> {
        let mut buf = vec![];
        let dest = dest.as_ref().join("index.html");
        if let Some(parent_dir) = dest.parent() {
            if !parent_dir.exists() {
                fs::create_dir_all(&parent_dir)?;
            }
        }

        get_tera().render_to(template, context, &mut buf)?;

        // Rewrite root path links with site url if and only if:
        // 1. in build run mode
        // 2. site url has a path
        if matches!(current_mode(), Some(Mode::Build)) {
            if let Some(Value::String(site_url)) =
                context.get("site").and_then(|site| site.get("url"))
            {
                let uri = site_url.parse::<Uri>().expect("Invalid site url.");
                // We don't need to rewrite links if the site url has a root path.
                if uri.path() != "/" {
                    let html = rewrite_html_base_url(&buf, site_url)?;
                    fs::write(dest, html)?;
                    return Ok(());
                }
            }
        }

        fs::write(dest, buf)?;
        Ok(())
    }

    // Render Atom feed
    fn render_atom_feed(context: Context, dest: impl AsRef<Path>) -> Result<()> {
        let mut buf = vec![];
        let dest = dest.as_ref().join("feed.xml");

        get_tera().render_to("feed.jinja", &context, &mut buf)?;
        fs::write(dest, buf)?;
        Ok(())
    }

    // Render sitemap.xml
    fn render_sitemap(context: Context, dest: impl AsRef<Path>) -> Result<()> {
        let mut buf = vec![];
        let dest = dest.as_ref().join("sitemap.xml");
        get_tera().render_to("sitemap.jinja", &context, &mut buf)?;
        fs::write(dest, buf)?;
        Ok(())
    }
}

impl ZineEngine {
    pub fn new<P: AsRef<Path>>(source: P, dest: P) -> Result<Self> {
        let dest = dest.as_ref().to_path_buf();
        if !dest.exists() {
            fs::create_dir_all(&dest)?;
        }
        Ok(ZineEngine {
            source: source.as_ref().to_path_buf(),
            dest,
        })
    }

    pub fn build(&self) -> Result<()> {
        let content =
            fs::read_to_string(&self.source.join(crate::ZINE_FILE)).with_context(|| {
                format!(
                    "Failed to parse root `zine.toml` of `{}`",
                    self.source.display()
                )
            })?;
        let mut zine = toml::from_str::<Zine>(&content)?;

        zine.parse(&self.source)?;

        // Init tera with parsed locale.
        let locale = zine.site.locale.as_deref().unwrap_or("en");
        init_tera(&self.source, locale);

        zine.render(Context::new(), &self.dest)?;
        #[cfg(debug_assertions)]
        println!("Zine engine: {:?}", zine);

        let mut feed_context = Context::new();
        feed_context.insert("site", &zine.site);
        feed_context.insert("entries", &zine.latest_feed_entries(20));
        feed_context.insert("generator_version", env!("CARGO_PKG_VERSION"));
        Render::render_atom_feed(feed_context, &self.dest)?;

        let mut sitemap_context = Context::new();
        sitemap_context.insert("site", &zine.site);
        sitemap_context.insert("entries", &zine.sitemap_entries());
        Render::render_sitemap(sitemap_context, &self.dest)?;

        Ok(())
    }
}

// A tera function to convert markdown into html.
fn markdown_to_html_fn(
    map: &std::collections::HashMap<String, serde_json::Value>,
) -> tera::Result<serde_json::Value> {
    use pulldown_cmark::*;

    struct HeadingRef<'a> {
        level: usize,
        id: Option<&'a str>,
    }

    if let Some(serde_json::Value::String(markdown)) = map.get("markdown") {
        let mut html = String::new();

        let parser_events_iter = Parser::new_ext(markdown, Options::all()).into_offset_iter();
        let ss = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();

        let mut events = vec![];
        let mut code_block_fenced = None;

        let mut heading_ref = None;
        for (event, _range) in parser_events_iter {
            match event {
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(name))) => {
                    code_block_fenced = Some(name);
                }
                Event::End(Tag::CodeBlock(_)) => {
                    code_block_fenced = None;
                }
                Event::Start(Tag::Image(_, src, title)) => {
                    // Add loading="lazy" attribute for markdown image.
                    events.push(Event::Html(
                        format!("<img src=\"{}\" alt=\"{}\" loading=\"lazy\">", src, title).into(),
                    ));
                }
                Event::Start(Tag::Heading(level, id, _)) => {
                    heading_ref = Some(HeadingRef {
                        level: level as usize,
                        // This id is parsed from the markdow heading part.
                        // Here is the syntax:
                        // `# Long title {#title}` parse the id: title
                        // See https://docs.rs/pulldown-cmark/latest/pulldown_cmark/struct.Options.html#associatedconstant.ENABLE_HEADING_ATTRIBUTES
                        id,
                    });
                }
                Event::End(Tag::Heading(..)) => {
                    heading_ref = None;
                }
                Event::Code(code) if code.starts_with('@') => {
                    if let Some(maybe_author_id) = code.strip_prefix('@') {
                        let data = data::get();
                        if let Some(author) = data.get_author_by_id(maybe_author_id) {
                            // Render author code UI.
                            let html = AuthorCode(author)
                                .render()
                                .expect("Render author code failed.");
                            events.push(Event::Html(html.into()));
                            continue;
                        }
                    }
                    events.push(Event::Code(code))
                }
                Event::Text(text) => {
                    if let Some(fenced) = code_block_fenced.as_ref() {
                        if ALL_CODE_BLOCKS.contains(&fenced.as_ref()) {
                            // Block in place to execute async task
                            let rendered_html = task::block_in_place(|| {
                                Handle::current()
                                    .block_on(async { render_code_block(fenced, &text).await })
                            });
                            if let Some(html) = rendered_html {
                                events.push(Event::Html(html.into()));
                                continue;
                            }
                        } else if let Some(syntax) = ss.find_syntax_by_extension(fenced) {
                            // Syntax highlight
                            let html = highlighted_html_for_string(
                                &text,
                                &ss,
                                syntax,
                                &ts.themes["InspiredGitHub"],
                            );
                            events.push(Event::Html(html.into()));
                            continue;
                        } else {
                            events.push(Event::Html(format!("<pre>{}</pre>", text).into()));
                            continue;
                        }
                    }

                    // Render heading anchor link.
                    if let Some(heading_ref) = heading_ref.as_ref() {
                        let mut context = Context::new();
                        context.insert("level", &heading_ref.level);
                        // Fallback to raw text as the anchor id if the user didn't specify an id.
                        context.insert("id", heading_ref.id.unwrap_or_else(|| text.as_ref()));
                        context.insert("text", &text.as_ref());
                        let html = get_tera()
                            .render("_anchor-link.jinja", &context)
                            .expect("Render anchor link failed.");

                        events.push(Event::Html(html.into()));
                        continue;
                    }

                    // Not a code block inside text, or the code block's fenced is unsupported.
                    // We still need record this text event.
                    events.push(Event::Text(text))
                }
                _ => {
                    events.push(event);
                }
            }
        }
        html::push_html(&mut html, events.into_iter());
        Ok(serde_json::Value::String(html))
    } else {
        Ok(serde_json::Value::Array(vec![]))
    }
}

fn get_author_fn(
    map: &std::collections::HashMap<String, serde_json::Value>,
) -> tera::Result<serde_json::Value> {
    if let Some(serde_json::Value::String(author_id)) = map.get("id") {
        let data = data::get();
        let author = data.get_author_by_id(author_id);
        Ok(serde_json::to_value(&author)?)
    } else {
        Ok(serde_json::Value::Null)
    }
}
