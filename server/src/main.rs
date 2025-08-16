use std::sync::Arc;

use common::net::Packet;
use null_talk_server::{
    ServerConfig,
    handlers::{handle_client, task::start_writer_task},
};
use tokio::{
    net::TcpListener,
    sync::{Mutex as AsyncMutex, mpsc},
};

#[tokio::main]
async fn main() {
    let config = match ServerConfig::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load config: {:?}", e);
            return;
        }
    };
    println!("🔧 Configuration Loaded");

    let server_address = config.get_addr();
    let listener = TcpListener::bind(&server_address).await.unwrap();
    println!("🚀 Server listening on {}", &server_address);

    let (tx, rx) = mpsc::unbounded_channel::<Packet>();
    let _ = start_writer_task(rx).await;

    let sender: Arc<AsyncMutex<mpsc::UnboundedSender<Packet>>> = Arc::new(AsyncMutex::new(tx));
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let sd_clone = sender.clone();
                tokio::spawn(async move { handle_client(stream, sd_clone).await });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {:?}", e);
                continue;
            }
        };
    }
}
