use serde::Serialize;
use time::Date;

use crate::entity::AuthorName;

#[derive(Serialize)]
pub struct FeedEntry<'a> {
    pub title: &'a String,
    pub url: String,
    pub content: &'a String,
    pub author: &'a Option<AuthorName>,
    #[serde(with = "crate::helpers::serde_date")]
    pub date: &'a Date,
}
