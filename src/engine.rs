use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use tera::{Context, Tera};
use tokio::{runtime::Handle, task};

use crate::{
    code_blocks::{render_code_block, ALL_CODE_BLOCKS},
    entity::{Entity, Zine},
};

static TERA: Lazy<RwLock<Tera>> = Lazy::new(|| {
    #[cfg(debug_assertions)]
    let mut tera = Tera::new("templates/*.jinja").expect("Invalid template dir.");

    #[cfg(not(debug_assertions))]
    let mut tera = Tera::default();
    #[cfg(not(debug_assertions))]
    tera.add_raw_templates(vec![
        ("_meta.jinja", include_str!("../templates/_meta.jinja")),
        ("base.jinja", include_str!("../templates/base.jinja")),
        ("index.jinja", include_str!("../templates/index.jinja")),
        ("season.jinja", include_str!("../templates/season.jinja")),
        ("article.jinja", include_str!("../templates/article.jinja")),
        ("page.jinja", include_str!("../templates/page.jinja")),
        ("feed.jinja", include_str!("../templates/feed.jinja")),
    ])
    .unwrap();
    tera.register_function("featured", featured_fn);
    tera.register_function("markdown_to_html", markdown_to_html_fn);
    RwLock::new(tera)
});

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

        TERA.read().render_to(template, context, &mut buf)?;
        fs::write(dest, buf)?;
        Ok(())
    }

    // Render Atom feed
    fn render_atom_feed(context: Context, dest: impl AsRef<Path>) -> Result<()> {
        let mut buf = vec![];
        let dest = dest.as_ref().join("feed.xml");

        TERA.read().render_to("feed.jinja", &context, &mut buf)?;
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
        #[cfg(debug_assertions)]
        {
            // Full realod tera to load templates dynamically.
            TERA.write().full_reload()?;
        }

        let content = fs::read_to_string(&self.source.join(crate::ZINE_FILE))?;
        let mut zine = toml::from_str::<Zine>(&content)?;

        zine.parse(&self.source)?;
        zine.render(Context::new(), &self.dest)?;
        #[cfg(debug_assertions)]
        println!("Zine engine: {:?}", zine);

        let mut feed_context = Context::new();
        feed_context.insert("site", &zine.site);
        feed_context.insert("entries", &zine.latest_feed_entries(20));
        Render::render_atom_feed(feed_context, &self.dest)?;
        Ok(())
    }
}

// A tera function to filter featured articles.
fn featured_fn(
    map: &std::collections::HashMap<String, serde_json::Value>,
) -> tera::Result<serde_json::Value> {
    if let Some(serde_json::Value::Array(articles)) = map.get("articles") {
        Ok(serde_json::Value::Array(
            articles
                .iter()
                .filter(|article| article.get("featured") == Some(&serde_json::Value::Bool(true)))
                .cloned()
                .collect(),
        ))
    } else {
        Ok(serde_json::Value::Array(vec![]))
    }
}

// A tera function to convert markdown into html.
fn markdown_to_html_fn(
    map: &std::collections::HashMap<String, serde_json::Value>,
) -> tera::Result<serde_json::Value> {
    use pulldown_cmark::*;

    if let Some(serde_json::Value::String(markdown)) = map.get("markdown") {
        let mut html = String::new();

        let parser_events_iter = Parser::new_ext(markdown, Options::all()).into_offset_iter();
        let mut events = vec![];
        let mut code_block_fenced = None;
        for (event, _range) in parser_events_iter {
            match event {
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(name)))
                    if ALL_CODE_BLOCKS.contains(&name.as_ref()) =>
                {
                    code_block_fenced = Some(name);
                }
                Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(name)))
                    if ALL_CODE_BLOCKS.contains(&name.as_ref()) =>
                {
                    code_block_fenced = None;
                }
                Event::Text(text) => {
                    if let Some(fenced) = code_block_fenced.as_ref() {
                        // Block in place to execute async task
                        let rendered_html = task::block_in_place(|| {
                            Handle::current()
                                .block_on(async { render_code_block(fenced, &text).await })
                        });
                        if let Some(html) = rendered_html {
                            events.push(Event::Html(html.into()));
                            continue;
                        }
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
