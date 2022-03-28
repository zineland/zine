use anyhow::Result;
use hyper::{
    body::{self, Buf},
    Client, Uri,
};
use hyper_tls::HttpsConnector;
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::{borrow::Cow, fs, io::Read, path::Path};

use html5ever::{
    parse_document, tendril::TendrilSink, tree_builder::TreeBuilderOpts, Attribute, ParseOpts,
};
use markup5ever_rcdom::{Handle, NodeData, RcDom};

use crate::meta::Meta;

pub async fn fetch_url(url: &str) -> Result<Meta<'_>> {
    let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
    let resp = client.get(url.parse::<Uri>()?).await?;
    let bytes = body::to_bytes(resp.into_body()).await?;
    let meta = parse_html_meta(bytes.reader());

    Ok(meta)
}

/// Parse HTML ['Meta`] from `html`.
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
                let attrs = attrs.borrow();
                match get_attribute(&*attrs, "name").or_else(|| get_attribute(&*attrs, "property"))
                {
                    Some("description" | "og:description" | "twitter:description")
                        if meta.description.is_empty() =>
                    {
                        if let Some(description) = get_attribute(&*attrs, "content") {
                            meta.description = Cow::Owned(description.trim().to_owned());
                        }
                    }
                    Some("og:title" | "twitter:title") if meta.title.is_empty() => {
                        if let Some(title) = get_attribute(&*attrs, "content") {
                            meta.title = Cow::Owned(title.trim().to_owned());
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

/// Copy directory recursively. 
/// Note: the empty directory is ignored.
pub fn copy_dir(source: &Path, dest: &Path) -> Result<()> {
    let source_parent = source.parent().expect("Can not copy the root dir");
    walkdir::WalkDir::new(source)
        .into_iter()
        .par_bridge()
        .try_for_each(|entry| {
            let entry = entry?;
            let path = entry.path();
            // `path` would be a file or directory. However, we are
            // in a rayon's parallel thread, there is no guarantee
            // that parent directory iterated before the file.
            // So we just ignore the `path.is_dir()` case, when coming
            // across the first file we'll create the parent directory.
            if path.is_file() {
                if let Some(parent) = path.parent() {
                    let dest_parent = dest.join(parent.strip_prefix(source_parent)?);
                    if !dest_parent.exists() {
                        // Create the same dir concurrently is ok according to the docs.
                        fs::create_dir_all(dest_parent)?;
                    }
                }
                let to = dest.join(path.strip_prefix(source_parent)?);
                fs::copy(path, to)?;
            }

            anyhow::Ok(())
        })?;
    Ok(())
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
