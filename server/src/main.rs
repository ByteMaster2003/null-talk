use dashmap::DashMap;
use server::{data::PeerMap, handlers::handle_client};
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(addr.clone()).await?;

    let peers: PeerMap = Arc::new(DashMap::new());

    println!("server is running on: {}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let peers_clone = peers.clone();

        tokio::spawn(async move { handle_client(stream, peers_clone).await });
    }
}
