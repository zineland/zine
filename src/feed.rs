use serde::Serialize;
use time::Date;

use crate::entity::AuthorId;

#[derive(Serialize)]
pub struct FeedEntry<'a> {
    pub title: &'a String,
    pub url: String,
    pub content: &'a String,
    pub author: &'a Option<AuthorId>,
    #[serde(with = "genkit::helpers::serde_date::options")]
    pub date: Option<Date>,
}
