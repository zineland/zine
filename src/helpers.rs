use anyhow::Result;
use hyper::{
    body::{self, Buf},
    Client, Uri,
};
use hyper_tls::HttpsConnector;
use std::io::Read;

use html5ever::{
    parse_document, tendril::TendrilSink, tree_builder::TreeBuilderOpts, Attribute, ParseOpts,
};
use markup5ever_rcdom::{Handle, NodeData, RcDom};

/// The meta info parse from HTML page, mainly including: `title`, `description`.
#[derive(Debug, Default)]
pub struct Meta {
    pub title: String,
    pub description: String,
}

impl Meta {
    pub fn is_filled(&self) -> bool {
        !self.title.is_empty() && !self.description.is_empty()
    }

    fn truncate(&mut self) {
        self.title.truncate(200);
        self.description.truncate(200);
    }
}

pub async fn fetch_url(url: &str) -> Result<Meta> {
    let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
    let resp = client.get(url.parse::<Uri>()?).await?;
    let bytes = body::to_bytes(resp.into_body()).await?;
    let meta = parse_html_meta(bytes.reader());

    Ok(meta)
}

/// Parse HTML ['Meta`] from `html`.
pub fn parse_html_meta<R: Read>(mut html: R) -> Meta {
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
                let attrs = attrs.borrow();
                match get_attribute(&*attrs, "name").or_else(|| get_attribute(&*attrs, "property"))
                {
                    Some("description" | "og:description" | "twitter:description")
                        if meta.description.is_empty() =>
                    {
                        if let Some(description) = get_attribute(&*attrs, "content") {
                            meta.description = description.trim().to_owned();
                        }
                    }
                    Some("og:title" | "twitter:title") if meta.title.is_empty() => {
                        if let Some(title) = get_attribute(&*attrs, "content") {
                            meta.title = title.trim().to_owned();
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
                meta.title = title.trim().to_owned();
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

/// A serde module to serialize and deserialize [`time::Date`] type.
pub mod serde_date {
    use serde::{de, Serialize, Serializer};
    use time::{format_description, Date};

    pub fn serialize<S: Serializer>(date: &Date, serializer: S) -> Result<S::Ok, S::Error> {
        let format = format_description::parse("[year]-[month]-[day]").expect("Shouldn't happen");
        date.format(&format)
            .expect("Serialize date error")
            .serialize(serializer)
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Date, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_any(DateVisitor)
    }

    struct DateVisitor;

    impl<'de> de::Visitor<'de> for DateVisitor {
        type Value = Date;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a date value like YYYY-MM-dd")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let format =
                format_description::parse("[year]-[month]-[day]").expect("Shouldn't happen");
            Ok(Date::parse(v, &format)
                .unwrap_or_else(|_| panic!("The date value {} is invalid", &v)))
        }
    }
}
