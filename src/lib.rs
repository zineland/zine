use time::Date;

pub struct ZineSite {
    name: String,
    logo: String,
    title: String,
    description: Option<String>,
}
pub struct Season {
    slug: String,
    number: u32,
    title: String,
    summary: Option<String>,
    articles: Vec<Article>,
}

pub struct Article {
    slug: String,
    title: String,
    author: Option<String>,
    content: String,
    pub_date: Date,
    publish: bool,
}

pub struct Page {
    slug: String,
    name: String,
    content: String,
}