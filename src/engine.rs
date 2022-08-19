use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    code_blocks::{is_custom_code_block, render_code_block, AuthorCode, CodeBlock},
    current_mode, data,
    entity::{Entity, MarkdownConfig, Zine},
    html::rewrite_html_base_url,
    locales::FluentLoader,
    markdown::{markdown_to_html, MarkdownVistor},
    Mode,
};

use anyhow::{Context as _, Result};
use hyper::Uri;
use once_cell::sync::Lazy;
use once_cell::sync::OnceCell;
use pulldown_cmark::*;
use serde_json::Value;
use syntect::{
    dumps::from_binary, highlighting::ThemeSet, html::highlighted_html_for_string,
    parsing::SyntaxSet,
};
use tera::{Context, Function, Tera};
use tokio::{runtime::Handle, task};

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| {
    let syntax_set: SyntaxSet =
        from_binary(include_bytes!("../sublime/syntaxes/newlines.packdump"));
    syntax_set
});
static THEME_SET: Lazy<ThemeSet> = Lazy::new(|| {
    let theme_set: ThemeSet = from_binary(include_bytes!("../sublime/themes/all.themedump"));
    theme_set
});
static TERA: OnceCell<parking_lot::RwLock<Tera>> = OnceCell::new();

fn init_tera(source: &Path, zine: &Zine) {
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
        tera.register_function("get_author", get_author_fn);

        parking_lot::RwLock::new(tera)
    });

    let locale = zine.site.locale.as_deref().unwrap_or("en");
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
    tera.register_function(
        "markdown_to_html",
        MarkdownRender {
            markdown_config: zine.markdown_config.clone(),
        },
    );
    tera.register_function("fluent", FluentLoader::new(source, locale));
}

fn get_tera() -> parking_lot::RwLockReadGuard<'static, Tera> {
    TERA.get().expect("Tera haven't initialized").read()
}

#[derive(Debug)]
pub struct ZineEngine {
    source: PathBuf,
    dest: PathBuf,
}

struct MarkdownRender {
    markdown_config: MarkdownConfig,
}

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
    if matches!(current_mode(), Mode::Build) {
        if let Some(Value::String(site_url)) = context.get("site").and_then(|site| site.get("url"))
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

/// Render raw template.
pub fn render_str(raw_template: &str, context: &Context) -> Result<String> {
    let mut tera = TERA.get().expect("Tera haven't initialized").write();
    let r = tera.render_str(raw_template, context)?;
    Ok(r)
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

        init_tera(&self.source, &zine);

        zine.render(Context::new(), &self.dest)?;
        #[cfg(debug_assertions)]
        println!("Zine engine: {:?}", zine);

        let mut feed_context = Context::new();
        feed_context.insert("site", &zine.site);
        feed_context.insert("entries", &zine.latest_feed_entries(20));
        feed_context.insert("generator_version", env!("CARGO_PKG_VERSION"));
        render_atom_feed(feed_context, &self.dest)?;

        let mut sitemap_context = Context::new();
        sitemap_context.insert("site", &zine.site);
        sitemap_context.insert("entries", &zine.sitemap_entries());
        render_sitemap(sitemap_context, &self.dest)?;

        Ok(())
    }
}

struct HeadingRef<'a> {
    level: usize,
    id: Option<&'a str>,
}

struct Vistor<'a> {
    markdown_config: &'a MarkdownConfig,
    code_block_fenced: Option<CowStr<'a>>,
    heading_ref: Option<HeadingRef<'a>>,
}

impl<'a> Vistor<'a> {
    fn new(markdown_config: &'a MarkdownConfig) -> Self {
        Vistor {
            markdown_config,
            code_block_fenced: None,
            heading_ref: None,
        }
    }

    fn highlight_syntax(&self, lang: &str, text: &str) -> String {
        let theme = match THEME_SET.themes.get(&self.markdown_config.highlight_theme) {
            Some(theme) => theme,
            None => panic!(
                "No theme: `{}` founded",
                self.markdown_config.highlight_theme
            ),
        };

        let syntax = SYNTAX_SET
            .find_syntax_by_token(lang)
            // Fallback to plain text if code block not supported
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());
        highlighted_html_for_string(text, &SYNTAX_SET, syntax, theme)
    }
}

impl<'a, 'b: 'a> MarkdownVistor<'b> for Vistor<'a> {
    fn visit_start_tag(&mut self, tag: Tag<'b>) -> Option<Event<'static>> {
        match tag {
            Tag::CodeBlock(CodeBlockKind::Fenced(name)) => {
                self.code_block_fenced = Some(name);
            }
            Tag::Image(_, src, title) => {
                // Add loading="lazy" attribute for markdown image.
                return Some(Event::Html(
                    format!("<img src=\"{}\" alt=\"{}\" loading=\"lazy\">", src, title).into(),
                ));
            }
            Tag::Heading(level, id, _) => {
                self.heading_ref = Some(HeadingRef {
                    level: level as usize,
                    // This id is parsed from the markdow heading part.
                    // Here is the syntax:
                    // `# Long title {#title}` parse the id: title
                    // See https://docs.rs/pulldown-cmark/latest/pulldown_cmark/struct.Options.html#associatedconstant.ENABLE_HEADING_ATTRIBUTES
                    id,
                });
            }
            _ => {}
        }
        None
    }

    fn visit_end_tag(&mut self, tag: Tag<'_>) -> Option<Event<'static>> {
        match tag {
            Tag::CodeBlock(_) => {
                self.code_block_fenced = None;
            }
            Tag::Heading(..) => {
                self.heading_ref = None;
            }
            _ => {}
        }
        None
    }

    fn visit_text(&mut self, text: &CowStr<'b>) -> Option<Event<'static>> {
        if let Some(fenced) = self.code_block_fenced.as_ref() {
            if is_custom_code_block(fenced.as_ref()) {
                // Block in place to execute async task
                let rendered_html = task::block_in_place(|| {
                    Handle::current().block_on(async { render_code_block(fenced, text).await })
                });
                if let Some(html) = rendered_html {
                    return Some(Event::Html(html.into()));
                }
            } else if self.markdown_config.highlight_code {
                // Syntax highlight
                let html = self.highlight_syntax(fenced, text);
                return Some(Event::Html(html.into()));
            } else {
                return Some(Event::Html(format!("<pre>{}</pre>", text).into()));
            }
        }

        // Render heading anchor link.
        if let Some(heading_ref) = self.heading_ref.as_ref() {
            let mut context = Context::new();
            context.insert("level", &heading_ref.level);
            // Fallback to raw text as the anchor id if the user didn't specify an id.
            context.insert("id", heading_ref.id.unwrap_or_else(|| text.as_ref()));
            context.insert("text", &text.as_ref());
            let html = get_tera()
                .render("_anchor-link.jinja", &context)
                .expect("Render anchor link failed.");

            return Some(Event::Html(html.into()));
        }

        None
    }

    fn visit_code(&mut self, code: &CowStr<'b>) -> Option<Event<'static>> {
        if let Some(maybe_author_id) = code.strip_prefix('@') {
            let data = data::read();
            if let Some(author) = data.get_author_by_id(maybe_author_id) {
                // Render author code UI.
                let html = AuthorCode(author)
                    .render()
                    .expect("Render author code failed.");
                return Some(Event::Html(html.into()));
            }
        }
        None
    }
}

// A tera function to convert markdown into html.
impl Function for MarkdownRender {
    fn call(&self, map: &HashMap<String, Value>) -> tera::Result<Value> {
        if let Some(Value::String(markdown)) = map.get("markdown") {
            let html = markdown_to_html(markdown, Vistor::new(&self.markdown_config));
            Ok(Value::String(html))
        } else {
            Ok(Value::Array(vec![]))
        }
    }
}

fn get_author_fn(map: &HashMap<String, Value>) -> tera::Result<Value> {
    if let Some(Value::String(author_id)) = map.get("id") {
        let data = data::read();
        let author = data.get_author_by_id(author_id);
        Ok(serde_json::to_value(&author)?)
    } else {
        Ok(Value::Null)
    }
}
