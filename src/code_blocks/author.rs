use std::fmt::Write;

use genkit::CodeBlock;

use crate::entity::Author;

/// The author code is designed to render the avatar-name link on the markdown page.
///
/// The syntax is very simple, just write like this `@author_id`.
/// If the `author_id` is declared in the `[authors]` table of the root `zine.toml`,
/// it will render the UI as expected, otherwise it fallback into the raw code UI.
pub struct AuthorCode<'a>(pub &'a Author);

impl<'a> CodeBlock for AuthorCode<'a> {
    fn render(&self) -> anyhow::Result<String> {
        let mut html = String::new();

        let author = self.0;
        writeln!(
            &mut html,
            r#"<a class="author-code" href="/@{}">"#,
            author.id,
        )?;
        if let Some(avatar) = author.avatar.as_ref() {
            writeln!(
                &mut html,
                r#"<img src="{}" alt="avatar" loading="lazy">"#,
                avatar,
            )?;
        }
        writeln!(
            &mut html,
            r#"<span>{}</span>"#,
            author.name.as_ref().unwrap()
        )?;
        writeln!(&mut html, r#"</a>"#)?;
        Ok(html)
    }
}
