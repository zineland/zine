use anyhow::Result;
use std::{borrow::Cow, io::Read};

use html5ever::{
    parse_document, tendril::TendrilSink, tree_builder::TreeBuilderOpts, Attribute, ParseOpts,
};
use lol_html::{element, html_content::Element, HtmlRewriter, Settings};
use markup5ever_rcdom::{Handle, NodeData, RcDom};

use serde::Serialize;

use crate::helpers;

/// The meta info of the HTML page.
#[derive(Debug, Default, Serialize)]
pub struct Meta<'a> {
    pub title: Cow<'a, str>,
    pub description: Cow<'a, str>,
    pub url: Option<Cow<'a, str>>,
    pub image: Option<Cow<'a, str>>,
}

impl<'a> Meta<'a> {
    pub fn truncate(&mut self) {
        self.title.to_mut().truncate(200);
        self.description.to_mut().truncate(200);
    }
}

/// Rewrite root path URL in `raw_html` with `site_url` and `cdn_url`.
pub fn rewrite_html_base_url(
    raw_html: &[u8],
    site_url: Option<&str>,
    cdn_url: Option<&str>,
) -> Result<Vec<u8>> {
    let rewrite_url_in_attr = |el: &mut Element, attr_name: &str| {
        if let Some(attr) = el.get_attribute(attr_name) {
            let dest_url =
                if let (Some(attr), Some(cdn_url)) = (attr.strip_prefix("/static"), cdn_url) {
                    format!("{}{}", &cdn_url, attr)
                } else if let (true, Some(site_url)) = (attr.starts_with('/'), site_url) {
                    format!("{}{}", &site_url, attr)
                } else {
                    // no need to rewrite
                    return;
                };

            el.set_attribute(attr_name, &dest_url)
                .expect("Set attribute failed");
        }
    };

    let mut html = vec![];
    let mut html_rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                element!("a[href], link[rel=stylesheet][href]", |el| {
                    rewrite_url_in_attr(el, "href");
                    Ok(())
                }),
                element!(
                    "script[src], iframe[src], img[src], audio[src], video[src]",
                    |el| {
                        rewrite_url_in_attr(el, "src");
                        Ok(())
                    }
                ),
                // Rewrite background image url.
                element!("body>div.bg-primary.text-main", |el| {
                    if let Some(style) = el.get_attribute("style") {
                        let mut pairs = helpers::split_styles(&style);
                        let backgrond_image_url = match pairs.get("background-image") {
                            Some(value) if value.starts_with("url('/static") => {
                                if let Some(cdn_url) = cdn_url {
                                    value.replacen("/static", cdn_url, 1)
                                } else {
                                    return Ok(());
                                }
                            }
                            Some(value) if value.starts_with("url('/") => {
                                if let Some(site_url) = site_url {
                                    value.replacen('/', &format!("{site_url}/"), 1)
                                } else {
                                    return Ok(());
                                }
                            }
                            _ => {
                                // no need to rewrite
                                return Ok(());
                            }
                        };

                        pairs.insert("background-image", &backgrond_image_url);
                        let new_style = pairs.into_iter().map(|(k, v)| format!("{k}: {v}")).fold(
                            String::new(),
                            |mut acc, pair| {
                                acc.push_str(&pair);
                                acc.push(';');
                                acc
                            },
                        );
                        el.set_attribute("style", &new_style)
                            .expect("Rewrite background-image failed.")
                    }
                    Ok(())
                }),
                element!("meta[content]", |el| {
                    rewrite_url_in_attr(el, "content");
                    Ok(())
                }),
            ],
            ..Default::default()
        },
        |c: &[u8]| {
            html.extend_from_slice(c);
        },
    );
    html_rewriter.write(raw_html)?;

    Ok(html)
}

/// Parse HTML [`Meta`] from `html`.
pub fn parse_html_meta<'a, R: Read>(mut html: R) -> Meta<'a> {
    let parse_opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            scripting_enabled: false,
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let rc_dom = parse_document(RcDom::default(), parse_opts)
        .from_utf8()
        .read_from(&mut html)
        .unwrap();

    let mut meta = Meta::default();
    if let NodeData::Document = rc_dom.document.data {
        let children = rc_dom.document.children.borrow();
        for child in children.iter() {
            if walk(child, &mut meta, "html") {
                // Stop traverse.
                break;
            }
        }
    } else {
        walk(&rc_dom.document, &mut meta, "html");
    }
    meta.truncate();
    meta
}

// Walk html tree to parse [`Meta`].
// `super_node` is the current node we traversing in.
//
// Return true if we should stop traversing.
fn walk(handle: &Handle, meta: &mut Meta, super_node: &str) -> bool {
    fn get_attribute<'a>(attrs: &'a [Attribute], name: &'a str) -> Option<&'a str> {
        attrs.iter().find_map(|attr| {
            if attr.name.local.as_ref() == name {
                let value = attr.value.as_ref().trim();
                // Some value of attribute is empty, such as:
                // <meta property="og:title" content="" />
                if value.is_empty() {
                    None
                } else {
                    Some(value)
                }
            } else {
                None
            }
        })
    }

    if let NodeData::Element {
        ref name,
        ref attrs,
        ..
    } = &handle.data
    {
        match name.local.as_ref() {
            node_name @ ("html" | "head") => {
                let children = handle.children.borrow();
                for child in children.iter() {
                    if walk(child, meta, node_name) {
                        // Stop traverse.
                        return true;
                    }
                }
            }
            "meta" if super_node == "head" => {
                // <meta name="description" content="xxx"/>
                // get description value from attribute.
                let attrs = &*attrs.borrow();
                match get_attribute(attrs, "name").or_else(|| get_attribute(attrs, "property")) {
                    Some("description" | "og:description" | "twitter:description")
                        if meta.description.is_empty() =>
                    {
                        if let Some(description) = get_attribute(attrs, "content") {
                            meta.description = Cow::Owned(description.trim().to_owned());
                        }
                    }
                    Some("og:title" | "twitter:title") if meta.title.is_empty() => {
                        if let Some(title) = get_attribute(attrs, "content") {
                            meta.title = Cow::Owned(title.trim().to_owned());
                        }
                    }
                    Some("og:image" | "twitter:image") if meta.image.is_none() => {
                        if let Some(image) = get_attribute(attrs, "content") {
                            meta.image = Some(Cow::Owned(image.to_owned()));
                        }
                    }
                    // url
                    Some("og:url" | "twitter:url") if meta.url.is_none() => {
                        if let Some(url) = get_attribute(attrs, "content") {
                            meta.url = Some(Cow::Owned(url.to_owned()));
                        }
                    }
                    _ => {}
                }
            }
            "link" if super_node == "head" => {
                // TODO: Extract favicon from <link> tag
            }
            "title" if super_node == "head" => {
                // Extract <title> tag.
                // Some title tag may have multiple empty text child nodes,
                // we need handle this case:
                //   <title>
                //
                //       Rust Programming Language
                //
                //   </title>
                let title = handle
                    .children
                    .borrow()
                    .iter()
                    .filter_map(|h| match &h.data {
                        NodeData::Text { contents } => {
                            let contents = contents.borrow();
                            Some(contents.to_string())
                        }
                        _ => None,
                    })
                    .collect::<String>();
                meta.title = Cow::Owned(title.trim().to_owned());
            }
            _ => {}
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::{parse_html_meta, rewrite_html_base_url};
    use test_case::test_case;

    const SITE_URL: &str = "https://github.com";
    const CDN_URL: &str = "https://cdn-example.net";

    #[test_case(r#"<body><div class="bg-primary text-main" style="background-image: url('/test.png');"></div></body>"#)]
    fn test_rewrite_background_image_url(html: &str) {
        assert_eq!(
            String::from_utf8_lossy(
                &rewrite_html_base_url(html.as_bytes(), Some(SITE_URL), Some(CDN_URL)).unwrap()
            ),
            html.replace("/test.png", &format!("{}/test.png", SITE_URL))
        );
    }

    #[test_case(r#"<body><div class="bg-primary text-main" style="background-image: url('/static/test.png');"></div></body>"#)]
    // #[test_case(r#"<body><div class="bg-primary text-main" style="background-image: URL('/static/test.png');"></div></body>"#; "uppercase")]
    fn test_rewrite_cdn_background_image_url(html: &str) {
        assert_eq!(
            String::from_utf8_lossy(
                &rewrite_html_base_url(html.as_bytes(), Some(SITE_URL), Some(CDN_URL)).unwrap()
            ),
            html.replace("/static/test.png", &format!("{}/test.png", CDN_URL))
        );
    }

    #[test_case("<a href=\"{}\"></a>", "/"; "a1")]
    #[test_case("<a href=\"{}\"></a>", "/hello"; "a2")]
    #[test_case("<a href=\"{}\"></a>", "/hello/world"; "a3")]
    #[test_case("<link rel=\"stylesheet\" href=\"{}\" />", "/hello.css"; "link")]
    #[test_case("<img src=\"{}\" />", "/hello.png"; "img")]
    #[test_case("<script src=\"{}\" />", "/hello.js"; "script")]
    #[test_case("<audio src=\"{}\" />", "/hello.mp3"; "audio")]
    #[test_case("<video src=\"{}\" />", "/hello.mp4"; "video")]
    #[test_case("<iframe src=\"{}\"></iframe>", "/hello.html"; "iframe")]
    fn test_rewrite_html_base_url(html: &str, path: &str) {
        assert_eq!(
            String::from_utf8_lossy(
                &rewrite_html_base_url(
                    html.replace("{}", path).as_bytes(),
                    Some(SITE_URL),
                    Some(CDN_URL)
                )
                .unwrap()
            ),
            html.replace("{}", &format!("{}{}", SITE_URL, path))
        );
    }

    #[test_case("<a href=\"{}\"></a>", "/"; "a1")]
    #[test_case("<a href=\"{}\"></a>", "/hello"; "a2")]
    #[test_case("<a href=\"{}\"></a>", "/hello/world"; "a3")]
    #[test_case("<link rel=\"stylesheet\" src=\"{}\"/>", "/hello.css"; "link")]
    #[test_case("<img src=\"{}\"/>", "/hello.png"; "img")]
    #[test_case("<script src=\"{}\"/>", "/hello.js"; "script")]
    #[test_case("<audio src=\"{}\"/>", "/hello.mp3"; "audio")]
    #[test_case("<video src=\"{}\"/>", "/hello.mp4"; "video")]
    #[test_case("<iframe src=\"{}\"></iframe>", "/hello.html"; "iframe")]
    fn test_not_rewrite_html_base_url(html: &str, path: &str) {
        let whole_url = format!("{}{}", SITE_URL, path);
        assert_eq!(
            String::from_utf8_lossy(
                &rewrite_html_base_url(
                    html.replace("{}", &whole_url).as_bytes(),
                    Some(SITE_URL),
                    Some(CDN_URL)
                )
                .unwrap()
            ),
            html.replace("{}", &whole_url)
        );
    }

    #[test_case("<a href=\"{}\"></a>", "hello"; "a1")]
    #[test_case("<link rel=\"stylesheet\" src=\"{}\"/>", "hello.css"; "link")]
    #[test_case("<img src=\"{}\"/>", "hello.png"; "img")]
    #[test_case("<script src=\"{}\"/>", "hello.js"; "script")]
    #[test_case("<audio src=\"{}\"/>", "hello.mp3"; "audio")]
    #[test_case("<video src=\"{}\"/>", "hello.mp4"; "video")]
    #[test_case("<iframe src=\"{}\"></iframe>", "hello.html"; "iframe")]
    fn test_not_rewrite_html_base_url_relative_path(html: &str, path: &str) {
        assert_eq!(
            String::from_utf8_lossy(
                &rewrite_html_base_url(
                    html.replace("{}", path).as_bytes(),
                    Some(SITE_URL),
                    Some(CDN_URL)
                )
                .unwrap()
            ),
            html.replace("{}", path)
        );
    }

    #[test_case("<link rel=\"stylesheet\" href=\"{}\" />", "/static/hello.css"; "link")]
    #[test_case("<img src=\"{}\" />", "/static/hello.png"; "img")]
    #[test_case("<script src=\"{}\" />", "/static/hello.js"; "script")]
    #[test_case("<audio src=\"{}\" />", "/static/hello.mp3"; "audio")]
    #[test_case("<video src=\"{}\" />", "/static/hello.mp4"; "video")]
    #[test_case("<meta content=\"{}\" />", "/static/zine-placeholder.svg"; "meta")]
    fn test_rewrite_html_cdn_url(html: &str, path: &str) {
        assert_eq!(
            String::from_utf8_lossy(
                &rewrite_html_base_url(
                    html.replace("{}", path).as_bytes(),
                    Some(SITE_URL),
                    Some(CDN_URL)
                )
                .unwrap()
            ),
            html.replace("{}", &format!("{}{}", CDN_URL, &path[7..]))
        );
    }

    #[test_case("<link rel=\"stylesheet\" src=\"{}\"/>", "/static/hello.css"; "link")]
    #[test_case("<img src=\"{}\"/>", "/static/hello.png"; "img")]
    #[test_case("<script src=\"{}\"/>", "/static/hello.js"; "script")]
    #[test_case("<audio src=\"{}\"/>", "/static/hello.mp3"; "audio")]
    #[test_case("<video src=\"{}\"/>", "/static/hello.mp4"; "video")]
    #[test_case("<meta content=\"{}\" />", "/static/zine-placeholder.svg"; "meta")]
    fn test_not_rewrite_html_cdn_url(html: &str, path: &str) {
        let whole_url = format!("{}{}", CDN_URL, path);
        assert_eq!(
            String::from_utf8_lossy(
                &rewrite_html_base_url(
                    html.replace("{}", &whole_url).as_bytes(),
                    Some(SITE_URL),
                    Some(CDN_URL)
                )
                .unwrap()
            ),
            html.replace("{}", &whole_url)
        );
    }

    #[test_case("<link rel=\"stylesheet\" src=\"{}\"/>", "static/hello.css"; "link")]
    #[test_case("<img src=\"{}\"/>", "static/hello.png"; "img")]
    #[test_case("<script src=\"{}\"/>", "static/hello.js"; "script")]
    #[test_case("<audio src=\"{}\"/>", "static/hello.mp3"; "audio")]
    #[test_case("<video src=\"{}\"/>", "static/hello.mp4"; "video")]
    #[test_case("<meta content=\"{}\" />", "static/zine-placeholder.svg"; "meta")]
    fn test_not_rewrite_html_cdn_url_relative_path(html: &str, path: &str) {
        assert_eq!(
            String::from_utf8_lossy(
                &rewrite_html_base_url(
                    html.replace("{}", path).as_bytes(),
                    Some(SITE_URL),
                    Some(CDN_URL)
                )
                .unwrap()
            ),
            html.replace("{}", path)
        );
    }

    #[test]
    fn test_parse_html_meta1() {
        let html = r#"
<!DOCTYPE html><html lang="en" class="notranslate" translate="no">
<head>
<meta charset="utf-8">
<meta http-equiv="X-UA-Compatible" content="IE=edge">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>crates.io: Rust Package Registry</title>
<link rel="shortcut icon" href="/favicon.ico" type="image/x-icon">
<link rel="icon" href="/assets/cargo.png" type="image/png">
<meta name="google" content="notranslate">
<meta property="og:image" content="/assets/og-image.png">
<meta name="twitter:card" content="summary_large_image">
</head>
<body></body></html>
        "#;
        let meta = parse_html_meta(html.as_bytes());
        assert_eq!(meta.title, "crates.io: Rust Package Registry");
        assert_eq!(meta.description, "");
        assert_eq!(meta.url, None);
        assert_eq!(meta.image, Some("/assets/og-image.png".into()));
    }

    #[test]
    fn test_parse_html_meta2() {
        let html = r#"
        
<!DOCTYPE html><html lang="en" class="notranslate" translate="no"><head>
<meta charset="utf-8">
<meta http-equiv="X-UA-Compatible" content="IE=edge">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>crates.io: Rust Package Registry</title>
<link rel="shortcut icon" href="/favicon.ico" type="image/x-icon">
<link rel="icon" href="/assets/cargo.png" type="image/png">
<link rel="search" href="/opensearch.xml" type="application/opensearchdescription+xml" title="Cargo">

<meta property="og:image" content="/assets/og-image.png">
<meta name="twitter:card" content="summary_large_image">

<body>
</body></html>
        "#;
        let meta = parse_html_meta(html.as_bytes());
        assert_eq!(meta.title, "crates.io: Rust Package Registry");
        assert_eq!(meta.description, "",);
        assert_eq!(meta.url, None);
        assert_eq!(meta.image, Some("/assets/og-image.png".into()));
    }

    #[test]
    fn test_parse_html_meta3() {
        let html = r#"<!DOCTYPE html><html lang="en" class="notranslate" translate="no">
<head>
<meta charset="utf-8">
<meta http-equiv="X-UA-Compatible" content="IE=edge">
<meta name="viewport" content="width=device-width, initial-scale=1">
<link rel="shortcut icon" href="/favicon.ico" type="image/x-icon">
<link rel="icon" href="/assets/cargo.png" type="image/png">
<meta property="og:image" content="/assets/og-image.png">
<meta name="twitter:card" content="summary_large_image">
<title>crates.io: Rust Package Registry</title>
<meta name="description" content="crates.io is a Rust community effort to create a shared registry of crates.">

<meta property="og:url" content="https://crates.io/">
<meta name="twitter:url" content="https://crates.io/">

</head>

<body></body>
<footer>
<title>fake title</title>
</footer>
</html>
        "#;
        let meta = parse_html_meta(html.as_bytes());
        assert_eq!(meta.title, "crates.io: Rust Package Registry");
        assert_eq!(
            meta.description,
            "crates.io is a Rust community effort to create a shared registry of crates."
        );
        assert_eq!(meta.url, Some("https://crates.io/".into()));
        assert_eq!(meta.image, Some("/assets/og-image.png".into()));
    }

    #[test]
    fn test_parse_html_meta4() {
        let html = r#"<head>
        <meta charset="utf-8">
        <meta http-equiv="X-UA-Compatible" content="IE=edge">
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <link rel="shortcut icon" href="/favicon.ico" type="image/x-icon">
        <link rel="icon" href="/assets/cargo.png" type="image/png">
        <meta property="og:image" content="/assets/og-image.png">
        <meta name="twitter:card" content="summary_large_image">
        <title>crates.io: Rust Package Registry</title>
        <meta name="description" content="crates.io is a Rust community effort to create a shared registry of crates.">
        
        <meta property="og:url" content="https://crates.io/">
        <meta name="twitter:url" content="https://crates.io/">
        
        </head>"#;
        let meta = parse_html_meta(html.as_bytes());
        assert_eq!(meta.title, "crates.io: Rust Package Registry");
        assert_eq!(
            meta.description,
            "crates.io is a Rust community effort to create a shared registry of crates."
        );
        assert_eq!(meta.url, Some("https://crates.io/".into()));
        assert_eq!(meta.image, Some("/assets/og-image.png".into()));
    }
}
