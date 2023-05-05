use std::{fs, path::Path};

use crate::{current_mode, data, locales::FluentLoader, Mode, Zine};
use genkit::{helpers::copy_dir, html::rewrite_html_base_url, Context, Entity, Generator};

use anyhow::{Context as _, Result};
use hyper::Uri;
use minijinja::{context, value::Value as JinjaValue, Environment};
use serde::Serialize;
use serde_json::Value;

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

pub struct ZineGenerator;

impl Generator for ZineGenerator {
    type Entity = Zine;

    fn on_load(&self, source: &std::path::Path) -> Result<Self::Entity> {
        data::load();
        let (_source, zine) = crate::locate_root_zine_folder(std::fs::canonicalize(source)?)?
            .with_context(|| "Failed to find the root zine.toml file".to_string())?;
        Ok(zine)
    }

    fn on_reload(&self, source: &std::path::Path) -> Result<Self::Entity> {
        Zine::parse_from_toml(source)
    }

    fn get_markdown_config(&self, zine: &Self::Entity) -> Option<genkit::entity::MarkdownConfig> {
        Some(zine.markdown_config.clone())
    }

    fn on_extend_environment<'a>(
        &self,
        source: &std::path::Path,
        mut env: minijinja::Environment<'a>,
        zine: &'a Self::Entity,
    ) -> minijinja::Environment<'a> {
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

        env.add_function("get_author", get_author_function);
        let fluent_loader = FluentLoader::new(source, &zine.site.locale);
        env.add_function("fluent", move |key: &str, number: Option<i64>| -> String {
            fluent_loader.format(key, number)
        });
        env
    }

    fn on_render(
        &self,
        env: &Environment,
        context: Context,
        zine: &Self::Entity,
        source: &Path,
        dest: &Path,
    ) -> Result<()> {
        zine.render(env, context, dest)?;
        render_atom_feed(
            env,
            context! {
                site => &zine.site,
                entries => &zine.latest_feed_entries(20),
                generator_version => env!("CARGO_PKG_VERSION"),
            },
            dest,
        )?;
        render_sitemap(
            env,
            context! {
                site => &zine.site,
                entries => &zine.sitemap_entries(),
            },
            dest,
        )?;

        copy_static_assets(source, dest)?;
        Ok(())
    }
}

fn get_author_function(id: &str) -> JinjaValue {
    let data = data::read();
    let author = data.get_author_by_id(id);
    JinjaValue::from_serializable(&author)
}

fn copy_static_assets(source: &Path, dest: &Path) -> Result<()> {
    let static_dir = source.join("static");
    if static_dir.exists() {
        copy_dir(&static_dir, dest)?;
    }

    // Copy builtin static files into dest static dir.
    let dest_static_dir = dest.join("static");
    #[allow(clippy::needless_borrow)]
    fs::create_dir_all(&dest_static_dir)?;

    #[cfg(not(debug_assertions))]
    include_dir::include_dir!("static").extract(dest_static_dir)?;
    // Alwasy copy static directory in debug mode.
    #[cfg(debug_assertions)]
    copy_dir(Path::new("static"), dest)?;

    Ok(())
}
