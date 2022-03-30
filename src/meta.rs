use std::borrow::Cow;

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
        !self.title.is_empty() && !self.description.is_empty()
    }

    pub fn truncate(&mut self) {
        self.title.to_mut().truncate(200);
        self.description.to_mut().truncate(200);
    }
}
