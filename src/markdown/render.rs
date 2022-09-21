use crate::{
    code_blocks::{AuthorCode, CodeBlock, Fenced, InlineLink},
    data, engine,
    entity::MarkdownConfig,
};

use once_cell::sync::Lazy;
use pulldown_cmark::*;
use syntect::{
    dumps::from_binary, highlighting::ThemeSet, html::highlighted_html_for_string,
    parsing::SyntaxSet,
};
use tera::Context;
use tokio::{runtime::Handle, task};

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| {
    let syntax_set: SyntaxSet =
        from_binary(include_bytes!("../../sublime/syntaxes/newlines.packdump"));
    syntax_set
});
static THEME_SET: Lazy<ThemeSet> = Lazy::new(|| {
    let theme_set: ThemeSet = from_binary(include_bytes!("../../sublime/themes/all.themedump"));
    theme_set
});

/// Markdown html render.
pub struct MarkdownRender<'a> {
    markdown_config: &'a MarkdownConfig,
    code_block_fenced: Option<CowStr<'a>>,
    heading: Option<HeadingRef<'a>>,
}

struct HeadingRef<'a> {
    level: usize,
    // This id is parsed from the markdow heading part.
    // Here is the syntax:
    // `# Long title {#title}` parse the id: title
    // See https://docs.rs/pulldown-cmark/latest/pulldown_cmark/struct.Options.html#associatedconstant.ENABLE_HEADING_ATTRIBUTES
    id: Option<&'a str>,
    title: Vec<CowStr<'a>>,
    events: Vec<Event<'a>>,
}

impl<'a> HeadingRef<'a> {
    fn new(level: usize, id: Option<&'a str>) -> Self {
        HeadingRef {
            level,
            id,
            title: Vec::new(),
            events: Vec::new(),
        }
    }

    fn push_event(&mut self, event: Event<'a>) -> &mut Self {
        self.events.push(event);
        self
    }

    fn push_text(&mut self, text: CowStr<'a>) -> &mut Self {
        self.title.push(text);
        self
    }

    // Render heading anchor link.
    fn render(self) -> Event<'static> {
        let mut context = Context::new();
        context.insert("level", &self.level);
        let id = if let Some(id) = self.id {
            id.to_owned()
        } else {
            // Fallback to raw text as the anchor id if the user didn't specify an id.
            self.title.join("")
        };
        context.insert("id", &id);
        let mut heading = String::new();
        html::push_html(&mut heading, self.events.into_iter());
        context.insert("heading", &heading);

        let html = engine::get_tera()
            .render("_anchor-link.jinja", &context)
            .expect("Render anchor link failed.");
        Event::Html(html.into())
    }
}

impl<'a> MarkdownRender<'a> {
    pub fn new(markdown_config: &'a MarkdownConfig) -> Self {
        MarkdownRender {
            markdown_config,
            code_block_fenced: None,
            heading: None,
        }
    }

    fn highlight_syntax(&self, lang: &str, text: &str) -> String {
        let theme = match THEME_SET.themes.get(&self.markdown_config.highlight_theme) {
            Some(theme) => theme,
            None => panic!(
                "No theme: `{}` founded",
                self.markdown_config.highlight_theme
            ),
        };

        let syntax = SYNTAX_SET
            .find_syntax_by_token(lang)
            // Fallback to plain text if code block not supported
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());
        highlighted_html_for_string(text, &SYNTAX_SET, syntax, theme)
    }

    /// Render markdown to HTML.
    pub fn render_html(&mut self, markdown: &'a str) -> String {
        let parser_events_iter = Parser::new_ext(markdown, Options::all()).into_offset_iter();
        let events = parser_events_iter
            .into_iter()
            .filter_map(move |(event, _)| match event {
                Event::Start(tag) => self.visit_start_tag(&tag).resolve(|| Event::Start(tag)),
                Event::End(tag) => self.visit_end_tag(&tag).resolve(|| Event::End(tag)),
                Event::Code(code) => self.visit_code(&code).resolve(|| Event::Code(code)),
                Event::Text(text) => self
                    .visit_text(&text)
                    // Not a code block inside text, or the code block's fenced is unsupported.
                    // We still need record this text event.
                    .resolve(|| Event::Text(text)),
                _ => Some(event),
            });
        let mut html = String::new();
        html::push_html(&mut html, events);
        html
    }

    fn visit_start_tag(&mut self, tag: &Tag<'a>) -> Visiting {
        match tag {
            Tag::CodeBlock(CodeBlockKind::Fenced(name)) => {
                self.code_block_fenced = Some(name.clone());
                Visiting::Ignore
            }
            Tag::Image(_, src, title) => {
                // Add loading="lazy" attribute for markdown image.
                Visiting::Event(Event::Html(
                    format!("<img src=\"{}\" alt=\"{}\" loading=\"lazy\">", src, title).into(),
                ))
            }
            Tag::Heading(level, id, _) => {
                self.heading = Some(HeadingRef::new(*level as usize, *id));
                Visiting::Ignore
            }
            _ => {
                if let Some(heading) = self.heading.as_mut() {
                    heading.push_event(Event::Start(tag.to_owned()));
                    Visiting::Ignore
                } else {
                    Visiting::NotChanged
                }
            }
        }
    }

    fn visit_end_tag(&mut self, tag: &Tag<'a>) -> Visiting {
        match tag {
            Tag::CodeBlock(_) => {
                self.code_block_fenced = None;
                Visiting::Ignore
            }
            Tag::Heading(..) => {
                if let Some(heading) = self.heading.take() {
                    // Render heading event.
                    Visiting::Event(heading.render())
                } else {
                    Visiting::Ignore
                }
            }
            _ => {
                if let Some(heading) = self.heading.as_mut() {
                    heading.push_event(Event::End(tag.to_owned()));
                    Visiting::Ignore
                } else {
                    Visiting::NotChanged
                }
            }
        }
    }

    fn visit_text(&mut self, text: &CowStr<'a>) -> Visiting {
        if let Some(heading) = self.heading.as_mut() {
            heading
                .push_text(text.to_owned())
                .push_event(Event::Text(text.to_owned()));
            return Visiting::Ignore;
        }

        if let Some(input) = self.code_block_fenced.as_ref() {
            let fenced = Fenced::parse(input).unwrap();
            if fenced.is_custom_code_block() {
                // Block in place to execute async task
                let rendered_html = task::block_in_place(|| {
                    Handle::current().block_on(async { fenced.render_code_block(text).await })
                });
                if let Some(html) = rendered_html {
                    return Visiting::Event(Event::Html(html.into()));
                }
            } else if self.markdown_config.highlight_code {
                // Syntax highlight
                let html = self.highlight_syntax(fenced.name, text);
                return Visiting::Event(Event::Html(html.into()));
            } else {
                return Visiting::Event(Event::Html(format!("<pre>{}</pre>", text).into()));
            }
        }

        Visiting::NotChanged
    }

    fn visit_code(&mut self, code: &CowStr<'a>) -> Visiting {
        if let Some(heading) = self.heading.as_mut() {
            heading
                .push_text(code.to_owned())
                .push_event(Event::Code(code.to_owned()));
            return Visiting::Ignore;
        }

        if let Some(maybe_author_id) = code.strip_prefix('@') {
            let data = data::read();
            if let Some(author) = data.get_author_by_id(maybe_author_id) {
                // Render author code UI.
                let html = AuthorCode(author)
                    .render()
                    .expect("Render author code failed.");
                return Visiting::Event(Event::Html(html.into()));
            }
        } else if code.starts_with('/') {
            let data = data::read();
            if let Some(article) = data.get_article_by_path(code.as_ref()) {
                let html = InlineLink::new(&article.title, code, &article.cover)
                    .render()
                    .expect("Render inline linke failed.");
                return Visiting::Event(Event::Html(html.into()));
            }
        }
        Visiting::NotChanged
    }
}

/// The markdown visit result.
enum Visiting {
    /// A new event should be rendered.
    Event(Event<'static>),
    /// Nothing changed, still render the origin event.
    NotChanged,
    /// Don't render this event.
    Ignore,
}

impl Visiting {
    fn resolve<'a, F>(self, not_changed: F) -> Option<Event<'a>>
    where
        F: FnOnce() -> Event<'a>,
    {
        match self {
            Visiting::Event(event) => Some(event),
            Visiting::NotChanged => Some(not_changed()),
            Visiting::Ignore => None,
        }
    }
}
