use clap::Parser;
use dashmap::DashMap;
use server::{
    config,
    data::{self, PeerMap},
    net::{handle_client, tls::create_tls_acceptor},
    utils::types::Args,
};
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    config(args);
    let cfg = data::CONFIG.get().unwrap().clone();

    let addr = format!("0.0.0.0:{}", cfg.port);
    let peers: PeerMap = Arc::new(DashMap::new());
    
    let listener = TcpListener::bind(addr.clone()).await?;
    println!("🚀 TCP Server listening on {}", addr);

    // TLS check
    if cfg.tls {
        let cert_path = cfg.cert_path.unwrap();
        let key_path = cfg.key_path.unwrap();
        let acceptor = match create_tls_acceptor(&cert_path, &key_path).await {
            Ok(acceptor) => acceptor,
            Err(e) => {
                eprintln!("Failed to create TLS acceptor: {:?}", e);
                return Ok(());
            }
        };
        println!("🔐 TLS Success");

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let acceptor = acceptor.clone();
                    let peers_clone = peers.clone();

                    tokio::spawn(async move {
                        match acceptor.accept(stream).await {
                            Ok(tls_stream) => {
                                handle_client(Box::new(tls_stream), peers_clone).await
                            }
                            Err(e) => eprintln!("TLS handshake failed: {:?}", e),
                        }
                    });
                }
                Err(e) => eprintln!("Failed to accept connection: {:?}", e),
            }
        }
    } else {
        loop {
            let (stream, _) = listener.accept().await?;
            let peers_clone = peers.clone();

            tokio::spawn(async move { handle_client(Box::new(stream), peers_clone).await });
        }
    }
}
