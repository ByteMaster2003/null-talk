use common::net::Packet;
use null_talk_server::{
    ServerConfig,
    handlers::{handle_client, task::start_writer_task},
    net::create_tls_acceptor,
};
use std::sync::Arc;
use tokio::{
    net::TcpListener,
    sync::{Mutex as AsyncMutex, mpsc},
};

/// Main entry point for the server
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

    // Shared channel for communication
    let (tx, rx) = mpsc::unbounded_channel::<Packet>();
    let _ = start_writer_task(rx).await;
    let sender: Arc<AsyncMutex<mpsc::UnboundedSender<Packet>>> = Arc::new(AsyncMutex::new(tx));

    // TLS check
    if let Some(tls_cfg) = &config.tls {
        let acceptor = match create_tls_acceptor(tls_cfg).await {
            Ok(acceptor) => acceptor,
            Err(e) => {
                eprintln!("Failed to create TLS acceptor: {:?}", e);
                return;
            }
        };
        let listener = TcpListener::bind(&server_address).await.unwrap();
        println!("🔒 TLS Server listening on {}", &server_address);

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let acceptor = acceptor.clone();
                    let sd_clone = sender.clone();

                    tokio::spawn(async move {
                        match acceptor.accept(stream).await {
                            Ok(tls_stream) => handle_client(Box::new(tls_stream), sd_clone).await,
                            Err(e) => eprintln!("TLS handshake failed: {:?}", e),
                        }
                    });
                }
                Err(e) => eprintln!("Failed to accept connection: {:?}", e),
            }
        }
    } else {
        let listener = TcpListener::bind(&server_address).await.unwrap();
        println!("🚀 Plain TCP Server listening on {}", &server_address);

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let sd_clone = sender.clone();
                    tokio::spawn(async move { handle_client(Box::new(stream), sd_clone).await });
                }
                Err(e) => eprintln!("Failed to accept connection: {:?}", e),
            }
        }
    }
}
