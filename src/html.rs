use anyhow::Result;
use std::{borrow::Cow, io::Read};

use html5ever::{
    parse_document, tendril::TendrilSink, tree_builder::TreeBuilderOpts, Attribute, ParseOpts,
};
use lol_html::{element, html_content::Element, HtmlRewriter, Settings};
use markup5ever_rcdom::{Handle, NodeData, RcDom};

use serde::Serialize;

/// The meta info of the HTML page.
#[derive(Debug, Default, Serialize)]
pub struct Meta<'a> {
    pub title: Cow<'a, str>,
    pub description: Cow<'a, str>,
    pub url: Option<Cow<'a, str>>,
    pub image: Option<Cow<'a, str>>,
}

impl<'a> Meta<'a> {
    pub fn is_filled(&self) -> bool {
        !self.title.is_empty()
            && !self.description.is_empty()
            && matches!(&self.image, Some(image) if !image.is_empty())
    }

    pub fn truncate(&mut self) {
        self.title.to_mut().truncate(200);
        self.description.to_mut().truncate(200);
    }
}

/// Rewrite root path URL of static files in `raw_html` with `cdn_url`.
pub fn rewrite_html_cdn_url(raw_html: &[u8], cdn_url: &str) -> Result<Vec<u8>> {
    let rewrite_url_in_attr = |el: &mut Element, attr_name: &str| {
        if let Some(attr) = el.get_attribute(attr_name) {
            if attr.starts_with("/static") {
                el.set_attribute(attr_name, &format!("{}{}", &cdn_url, attr))
                    .expect("Set attribute failed");
            }
        }
    };

    let mut html = vec![];
    let mut html_rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                element!("link[rel=stylesheet][href]", |el| {
                    rewrite_url_in_attr(el, "href");
                    Ok(())
                }),
                element!("script[src], img[src], audio[src], video[src]", |el| {
                    rewrite_url_in_attr(el, "src");
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

/// Rewrite root path URL in `raw_html` with `base_url`.
pub fn rewrite_html_base_url(raw_html: &[u8], base_url: &str) -> Result<Vec<u8>> {
    let rewrite_url_in_attr = |el: &mut Element, attr_name: &str| {
        if let Some(attr) = el.get_attribute(attr_name) {
            if attr.starts_with('/') {
                el.set_attribute(attr_name, &format!("{}{}", &base_url, attr))
                    .expect("Set attribute failed");
            }
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
                    if let Some(attr) = el.get_attribute("style") {
                        if attr.starts_with("background-image: url('/") {
                            el.set_attribute(
                                "style",
                                &attr.replace(
                                    "background-image: url('",
                                    &format!("background-image: url('{}", base_url),
                                ),
                            )
                            .expect("Rewrite background-image failed.")
                        }
                    }
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
    walk(&rc_dom.document, &mut meta);
    meta.truncate();
    meta
}

// Walk html tree to parse [`Meta`].
fn walk(handle: &Handle, meta: &mut Meta) {
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
    } = handle.data
    {
        match name.local.as_ref() {
            "meta" => {
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
                    _ => {}
                }
            }
            "link" => {
                // TODO: Extract favicon from <link> tag
            }
            "title" => {
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
    let children = handle.children.borrow();
    for child in children.iter() {
        walk(child, meta);

        // If meta is filled, no need to walk.
        if meta.is_filled() {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::rewrite_html_base_url;
    use super::rewrite_html_cdn_url;
    use test_case::test_case;

    const BASE_URL: &str = "https://github.com";
    const CDN_URL: &str = "https://example-cdn.com";

    #[test_case(
        r#"
        <body class="h-full bg-secondary">
            <div class="bg-primary text-main" style="background-image: url('/test.png')"></div>
        </body>
        "#
    )]
    fn test_rewrite_background_image_url(html: &str) {
        assert_eq!(
            String::from_utf8_lossy(&rewrite_html_base_url(html.as_bytes(), BASE_URL).unwrap()),
            html.replace("/test.png", &format!("{}/test.png", BASE_URL))
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
                &rewrite_html_base_url(html.replace("{}", path).as_bytes(), BASE_URL).unwrap()
            ),
            html.replace("{}", &format!("{}{}", BASE_URL, path))
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
        let whole_url = format!("{}{}", BASE_URL, path);
        assert_eq!(
            String::from_utf8_lossy(
                &rewrite_html_base_url(html.replace("{}", &whole_url).as_bytes(), BASE_URL)
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
                &rewrite_html_base_url(html.replace("{}", path).as_bytes(), BASE_URL).unwrap()
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
                &rewrite_html_cdn_url(html.replace("{}", path).as_bytes(), CDN_URL).unwrap()
            ),
            html.replace("{}", &format!("{}{}", CDN_URL, path))
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
                &rewrite_html_cdn_url(html.replace("{}", &whole_url).as_bytes(), BASE_URL).unwrap()
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
                &rewrite_html_cdn_url(html.replace("{}", path).as_bytes(), CDN_URL).unwrap()
            ),
            html.replace("{}", path)
        );
    }
}
