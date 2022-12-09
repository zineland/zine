use futures::{sink::SinkExt, stream::StreamExt};
use hyper::{Body, Request, Response};
use hyper_tungstenite::{tungstenite, HyperWebsocket};
use tungstenite::Message;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Handle a WebSocket request.
pub async fn handle_request(mut request: Request<Body>) -> Result<Response<Body>, Error> {
    // Check if the request is a websocket upgrade request.
    if hyper_tungstenite::is_upgrade_request(&request) {
        let (response, websocket) = hyper_tungstenite::upgrade(&mut request, None)?;

        // Spawn a task to handle the websocket connection.
        tokio::spawn(async move {
            if let Err(e) = serve_websocket(websocket).await {
                eprintln!("Error in websocket connection: {}", e);
            }
        });

        // Return the response so the spawned future can continue.
        Ok(response)
    } else {
        Ok(Response::new(Body::from("Not a websocket request!")))
    }
}

// Handle a websocket connection.
async fn serve_websocket(websocket: HyperWebsocket) -> Result<(), Error> {
    let mut websocket = websocket.await?;
    while let Some(message) = websocket.next().await {
        if let Message::Text(msg) = message? {
            println!("Received text message: {}", msg);
            websocket.send(Message::text("reload")).await?;
        }
    }
    Ok(())
}
