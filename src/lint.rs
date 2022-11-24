use std::{collections::HashMap, path::Path};

use anyhow::Result;
use futures::future::try_join_all;
use hyper::{Client, Request};
use hyper_tls::HttpsConnector;

use crate::data;

pub async fn lint_zine_project<P: AsRef<Path>>(source: P) -> Result<()> {
    let tasks = {
        data::load(source);
        let guard = data::read();
        let url_previews = guard.get_all_previews();
        url_previews
            .iter()
            .map(|kv| {
                let (url, _) = kv.pair();
                check_url(url.to_owned())
            })
            .collect::<Vec<_>>()
    };

    let conditions =
        try_join_all(tasks)
            .await?
            .into_iter()
            .fold(
                HashMap::new(),
                |mut acc, (url, condition)| match condition {
                    UrlCondition::Normal => acc,
                    _ => {
                        let vec: &mut Vec<_> = acc.entry(condition).or_default();
                        vec.push(url);
                        acc
                    }
                },
            );

    if let Some(urls) = conditions.get(&UrlCondition::NotFound) {
        println!("\nThe following URLs are 404:");
        urls.iter().for_each(|url| println!("- {url}"));
    }
    if let Some(urls) = conditions.get(&UrlCondition::Redirected) {
        println!("\nThe following URLs have been redirected:");
        urls.iter().for_each(|url| println!("- {url}"));
    }
    if let Some(urls) = conditions.get(&UrlCondition::ServerError) {
        println!("\nThe following URLs have a server error:");
        urls.iter().for_each(|url| println!("- {url}"));
    }
    Ok(())
}

async fn check_url(url: String) -> Result<(String, UrlCondition)> {
    let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
    let req = Request::head(url.as_str()).body(hyper::Body::empty())?;
    let resp = client.request(req).await?;

    let status = resp.status();
    let condition = if status.as_u16() == 404 {
        UrlCondition::NotFound
    } else if status.is_redirection() {
        UrlCondition::Redirected
    } else if status.is_server_error() {
        UrlCondition::ServerError
    } else {
        UrlCondition::Normal
    };
    Ok((url, condition))
}

#[derive(Debug, Hash, PartialEq, Eq)]
enum UrlCondition {
    Normal,
    NotFound,
    Redirected,
    ServerError,
}
