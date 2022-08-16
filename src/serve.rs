use std::{env, io, net::SocketAddr, path::Path};

use crate::{build::watch_build, ZINE_BANNER};
use anyhow::Result;
use http_body::Full;
use hyper::{body::HttpBody, Response, StatusCode};
use tower::ServiceBuilder;
use tower_http::services::{fs::ServeFileSystemResponseBody, ServeDir};

// The temporal build dir, mainly for `zine serve` command.
static TEMP_ZINE_BUILD_DIR: &str = "__zine_build";

pub async fn run_serve(source: String, port: u16) -> Result<()> {
    let tmp_dir = env::temp_dir().join(TEMP_ZINE_BUILD_DIR);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let service = ServiceBuilder::new()
        .and_then(
            |response: Response<ServeFileSystemResponseBody>| async move {
                let response = if response.status() == StatusCode::NOT_FOUND {
                    let body = Full::from("404 Not Found")
                        .map_err(|err| match err {})
                        .boxed();
                    Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(body)
                        .unwrap()
                } else {
                    response.map(|body| body.boxed())
                };

                Ok::<_, io::Error>(response)
            },
        )
        .service(ServeDir::new(&tmp_dir));

    tokio::spawn(async move {
        watch_build(Path::new(&source), tmp_dir.as_path(), true)
            .await
            .unwrap();
    });

    println!("{}", ZINE_BANNER);
    println!("listening on http://{}", addr);
    hyper::Server::bind(&addr)
        .serve(tower::make::Shared::new(service))
        .await
        .expect("server error");
    Ok(())
}
