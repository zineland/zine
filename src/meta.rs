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
