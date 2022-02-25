use time::Date;

pub struct ZineSite {
    name: String,
    logo: String,
    title: String,
    description: Option<String>,
}
pub struct Season {
    id: u32,
    number: u32,
    summary: String,
    articles: Vec<Article>,
}

pub struct Article {
    season_id: u32,
    slug: String,
    title: String,
    author: Option<String>,
    content: String,
    pub_date: Date,
}

pub struct Page {
    slug: String,
    name: String,
    content: String,
}