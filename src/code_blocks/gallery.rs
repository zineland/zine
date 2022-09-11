use std::fmt::Write;

use super::CodeBlock;

pub struct GalleryBlock<'a> {
    images: Vec<&'a str>,
}

// enum GalleryMode {
//     Grid,
//     Slide,
// }

impl<'a> GalleryBlock<'a> {
    pub fn new(block: &'a str) -> Self {
        let images = block.lines().collect();
        GalleryBlock { images }
    }
}

impl<'a> CodeBlock for GalleryBlock<'a> {
    fn render(&self) -> anyhow::Result<String> {
        let mut html = String::new();

        writeln!(&mut html, r#"<div class="gallery">"#)?;
        for image in &self.images {
            writeln!(&mut html, r#"<p><img src="{}" /></p>"#, image)?;
        }
        writeln!(&mut html, r#"</div>"#)?;
        Ok(html)
    }
}
