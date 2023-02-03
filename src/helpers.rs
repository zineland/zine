use anyhow::Result;
use hyper::{
    body::{self, Buf},
    http::HeaderValue,
    Client, Request, Uri,
};
use hyper_tls::HttpsConnector;
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::{
    collections::HashMap,
    fs,
    io::{self, ErrorKind, Read},
    path::Path,
    process::Command,
};
use time::OffsetDateTime;

pub fn run_command(program: &str, args: &[&str]) -> Result<String, io::Error> {
    let out = Command::new(program).args(args).output()?;
    match out.status.success() {
        true => Ok(String::from_utf8(out.stdout).unwrap().trim().to_string()),
        false => Err(io::Error::new(
            ErrorKind::Other,
            format!("run command `{program} {}` failed.", args.join(" ")),
        )),
    }
}

pub fn get_date_of_today() ->  time::Date {
      OffsetDateTime::now_utc().date()
}

pub fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
    }
}

/// Split styles into string pair.
///
/// ```rust
/// use zine::helpers::split_styles;
///
/// let pair = split_styles("color: #abcdef; font-size: 14px; background-image: url('/test.png');");
/// assert_eq!(pair.get("color").unwrap(), &"#abcdef");
/// assert_eq!(pair.get("font-size").unwrap(), &"14px");
/// assert_eq!(pair.get("background-image").unwrap(), &"url('/test.png')");
/// assert_eq!(pair.get("width"), None);
///
/// let pair = split_styles("invalid");
/// assert!(pair.is_empty());
/// ```
pub fn split_styles(style: &str) -> HashMap<&str, &str> {
    style
        .split(';')
        .filter_map(|pair| {
            let mut v = pair.split(':').take(2);
            match (v.next(), v.next()) {
                (Some(key), Some(value)) => Some((key.trim(), value.trim())),
                _ => None,
            }
        })
        .collect::<HashMap<_, _>>()
}

pub async fn fetch_url(url: &str) -> Result<impl Read> {
    let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
    let mut req = Request::new(Default::default());
    *req.uri_mut() = url.parse::<Uri>()?;
    req.headers_mut().insert(
        "User-Agent",
        HeaderValue::from_static(
            "Mozilla/5.0 AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.0.0.0 Safari/537.36",
        ),
    );
    let resp = client.request(req).await?;
    if resp.status().is_redirection() {
        if let Some(location) = resp.headers().get("Location") {
            println!(
                "Warning: url `{url}` has been redirected to `{}`",
                location.to_str()?,
            );
        } else {
            println!("Warning: url `{url}` has been redirected");
        }
    }
    let bytes = body::to_bytes(resp.into_body()).await?;
    Ok(bytes.reader())
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
