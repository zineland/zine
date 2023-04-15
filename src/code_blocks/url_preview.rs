use std::{collections::HashMap, fmt::Write};

use anyhow::Result;

use crate::data::{self, PreviewEvent, UrlPreviewInfo};

use super::CodeBlock;

pub(super) struct UrlPreviewBlock<'a> {
    url: &'a str,
    info: UrlPreviewInfo,
    // Whether show the preview image. default to true.
    show_image: bool,
}

impl<'a> UrlPreviewBlock<'a> {
    pub(super) fn new(
        options: HashMap<String, &'a str>,
        url: &'a str,
        info: UrlPreviewInfo,
    ) -> Self {
        UrlPreviewBlock {
            url,
            info,
            show_image: options
                .get("image")
                .and_then(|v| str::parse::<bool>(v).ok())
                .unwrap_or(true),
        }
    }
}

impl<'a> CodeBlock for UrlPreviewBlock<'a> {
    fn render(&self) -> Result<String> {
        let mut html = String::new();
        writeln!(&mut html, r#"<div class="url-preview">"#)?;
        writeln!(&mut html, r#" <div>{}</div>"#, self.info.title)?;
        writeln!(&mut html, r#" <div>{}</div>"#, self.info.description)?;
        writeln!(&mut html, r#" <a href="{url}">{url}</a>"#, url = self.url)?;
        if self.show_image {
            if let Some(image) = self.info.image.as_ref().filter(|i| !i.is_empty()) {
                writeln!(&mut html, r#" <img src="{}" />"#, image)?;
            }
        }
        writeln!(&mut html, r#"</div>"#)?;
        Ok(html)
    }
}

pub(super) struct UrlPreviewError<'a>(pub &'a str, pub &'a str);

impl<'a> CodeBlock for UrlPreviewError<'a> {
    fn render(&self) -> Result<String> {
        let mut html = String::new();
        writeln!(&mut html, r#"<div class="url-preview">"#)?;
        writeln!(&mut html, r#" <div></div>"#)?;
        writeln!(&mut html, r#" <div>Url preview error: {}</div>"#, self.1)?;
        writeln!(&mut html, r#" <a href="{url}">{url}</a>"#, url = self.0)?;
        writeln!(&mut html, r#"</div>"#)?;
        Ok(html)
    }
}

/// Render the preview url if success, otherwise return preview error
/// to remind user we have error.
#[tokio::main(flavor = "current_thread")]
pub(crate) async fn render(url: &str, options: HashMap<String, &str>) -> Option<String> {
    let (first_preview, mut rx) = {
        // parking_lot RwLock guard isn't async-aware,
        // we should keep this guard drop in this scope.
        let data = data::read();
        if let Some(info) = data.get_preview(url) {
            let html = UrlPreviewBlock::new(options, url, info).render().unwrap();
            return Some(html);
        }

        data.preview_url(url)
    };
    rx.changed()
        .await
        .expect("URL preview watch channel receive failed.");
    let event = rx.borrow();
    match event.to_owned().expect("Url preview didn't initialized.") {
        PreviewEvent::Finished(info) => {
            let html = UrlPreviewBlock::new(options, url, info).render().unwrap();
            if first_preview {
                println!("URL previewed: {url}");
            }
            Some(html)
        }
        PreviewEvent::Failed(err) => {
            // Return a preview error block.
            Some(UrlPreviewError(url, &err).render().unwrap())
        }
    }
}
