use futures::{sink::SinkExt, stream::StreamExt};
use hyper::{Body, Request, Response};
use hyper_tungstenite::{tungstenite, HyperWebsocket};
use tungstenite::Message;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Handle a HTTP or WebSocket request.
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
        // Handle regular HTTP requests here.
        Ok(Response::new(Body::from("Hello HTTP!")))
    }
}

/// Handle a websocket connection.
async fn serve_websocket(websocket: HyperWebsocket) -> Result<(), Error> {
    let mut websocket = websocket.await?;
    while let Some(message) = websocket.next().await {
        match message? {
            Message::Text(msg) => {
                println!("Received text message: {}", msg);
                websocket
                    .send(Message::text("Thank you, come again."))
                    .await?;
            }
            Message::Binary(msg) => {
                println!("Received binary message: {:02X?}", msg);
                websocket
                    .send(Message::binary(b"Thank you, come again.".to_vec()))
                    .await?;
            }
            Message::Ping(msg) => {
                // No need to send a reply: tungstenite takes care of this for you.
                println!("Received ping message: {:02X?}", msg);
            }
            Message::Pong(msg) => {
                println!("Received pong message: {:02X?}", msg);
            }
            Message::Close(msg) => {
                // No need to send a reply: tungstenite takes care of this for you.
                if let Some(msg) = &msg {
                    println!(
                        "Received close message with code {} and message: {}",
                        msg.code, msg.reason
                    );
                } else {
                    println!("Received close message");
                }
            }
            Message::Frame(msg) => {
                unreachable!();
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let addr: std::net::SocketAddr = "[::1]:3000".parse()?;
    println!("Listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("listening on {}", addr);

    let mut http = hyper::server::conn::Http::new();
    http.http1_only(true);
    http.http1_keep_alive(true);

    loop {
        let (stream, _) = listener.accept().await?;
        let connection = http
            .serve_connection(stream, hyper::service::service_fn(handle_request))
            .with_upgrades();
        tokio::spawn(async move {
            if let Err(err) = connection.await {
                println!("Error serving HTTP connection: {:?}", err);
            }
        });
    }
}
