use serde::Serialize;
use time::Date;

#[derive(Serialize)]
pub struct FeedEntry<'a> {
    pub title: &'a String,
    pub url: String,
    pub content: &'a String,
    pub author: &'a Option<String>,
    #[serde(with = "crate::helps::serde_date")]
    pub date: &'a Date,
}
