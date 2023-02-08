use std::{
    env, fs,
    future::Future,
    io,
    net::SocketAddr,
    path::Path,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{build::watch_build, ZINE_BANNER};
use anyhow::Result;
use futures::SinkExt;
use hyper::{Body, Method, Request, Response, StatusCode};
use hyper_tungstenite::tungstenite::Message;
use tokio::sync::broadcast::{self, Sender};
use tower::Service;
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
    let (tx, _rx) = broadcast::channel(64);
    let serve_dir = ServeDir::new(&tmp_dir).fallback(FallbackService { tx: tx.clone() });

    tokio::spawn(async move {
        watch_build(Path::new(&source), tmp_dir.as_path(), true, Some(tx))
            .await
            .unwrap();
    });

    println!("{ZINE_BANNER}");
    println!("listening on http://{addr}");
    hyper::Server::bind(&addr)
        .serve(tower::make::Shared::new(serve_dir))
        .await
        .expect("server error");
    Ok(())
}

// A fallback service to handle websocket request and ServeDir's 404 request.
#[derive(Clone)]
struct FallbackService {
    tx: Sender<()>,
}

impl Service<Request<Body>> for FallbackService {
    type Response = Response<Body>;
    type Error = io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        let mut reload_rx = self.tx.subscribe();
        let fut = async move {
            let path = req.uri().path();
            match (req.method(), path) {
                (&Method::GET, "/live_reload") => {
                    // Check if the request is a websocket upgrade request.
                    if hyper_tungstenite::is_upgrade_request(&req) {
                        let (response, websocket) =
                            hyper_tungstenite::upgrade(&mut req, None).unwrap();

                        // Spawn a task to handle the websocket connection.
                        tokio::spawn(async move {
                            let mut websocket = websocket.await.unwrap();
                            while reload_rx.recv().await.is_ok() {
                                // Ignore the send failure, the reason could be: Broken pipe
                                let _ = websocket.send(Message::text("reload")).await;
                            }
                        });

                        // Return the response so the spawned future can continue.
                        Ok(response)
                    } else {
                        Ok(Response::new(Body::from("Not a websocket request!")))
                    }
                }
                _ => {
                    // Return 404 not found response.
                    let resp = Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::from("404 Not Found"))
                        .unwrap();
                    Ok(resp)
                }
            }
        };
        Box::pin(fut)
    }
}
