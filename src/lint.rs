use std::{collections::HashMap, path::Path};

use anyhow::Result;
use futures::future::try_join_all;
use hyper::{Client, Request};
use hyper_tls::HttpsConnector;

use crate::data;

/// Lint the zine project.
/// Return true if lint success.
pub async fn lint_zine_project<P: AsRef<Path>>(source: P) -> Result<bool> {
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

    let check_condition = |condition, statement: &str| {
        if let Some(urls) = conditions.get(&condition) {
            println!("\nThe following URLs {statement}:");
            urls.iter().for_each(|url| println!("- {url}"));
        }
    };
    check_condition(UrlCondition::NotFound, "are 404");
    check_condition(UrlCondition::Redirected, "have been redirected");
    check_condition(UrlCondition::ServerError, "have a server error");

    Ok(conditions.is_empty())
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
