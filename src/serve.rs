use std::{env, fs, io, net::SocketAddr, path::Path};

use crate::{build::watch_build, ws, ZINE_BANNER};
use anyhow::Result;
use hyper::{service::service_fn, Body, Method, Request, Response, StatusCode};
use tower_http::services::ServeDir;

// The temporal build dir, mainly for `zine serve` command.
static TEMP_ZINE_BUILD_DIR: &str = "__zine_build";

pub async fn run_serve(source: String, port: u16) -> Result<()> {
    let tmp_dir = env::temp_dir().join(TEMP_ZINE_BUILD_DIR);
    if tmp_dir.exists() {
        // Remove cached build directory to invalidate the old cache.
        fs::remove_dir_all(&tmp_dir)?;
    }

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let serve_dir = ServeDir::new(&tmp_dir).fallback(service_fn(handle_fallback_request));

    tokio::spawn(async move {
        watch_build(Path::new(&source), tmp_dir.as_path(), true)
            .await
            .unwrap();
    });

    println!("{}", ZINE_BANNER);
    println!("listening on http://{}", addr);
    hyper::Server::bind(&addr)
        .serve(tower::make::Shared::new(serve_dir))
        .await
        .expect("server error");
    Ok(())
}

async fn handle_fallback_request(
    req: Request<Body>,
) -> std::result::Result<Response<Body>, io::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/live_reload") => Ok(ws::handle_request(req).await.unwrap()),
        _ => {
            // Return 404 not found response.
            let resp = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("404 Not Found"))
                .unwrap();
            Ok(resp)
        }
    }
}
