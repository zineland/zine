use std::{env, net::SocketAddr, path::Path};

use crate::{build::watch_build, TEMP_ZINE_BUILD_DIR};
use anyhow::Result;
use tower_http::services::ServeDir;

pub async fn run_serve(source: String, port: u16) -> Result<()> {
    let tmp_dir = env::temp_dir().join(TEMP_ZINE_BUILD_DIR);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let service = ServeDir::new(&tmp_dir);
    tokio::spawn(async move {
        watch_build(Path::new(&source), tmp_dir.as_path(), true)
            .await
            .unwrap();
    });

    println!("listening on http://{}", addr.to_string());
    hyper::Server::bind(&addr)
        .serve(tower::make::Shared::new(service))
        .await
        .expect("server error");
    Ok(())
}
