use common::net::Packet;
use null_talk_server::{
    ServerConfig,
    handlers::{handle_client, task::start_writer_task},
};
use rustls::pki_types::PrivateKeyDer;
use std::{fs, io::BufReader, sync::Arc};
use tokio::{
    net::TcpListener,
    sync::{Mutex as AsyncMutex, mpsc},
};
use tokio_rustls::{TlsAcceptor, rustls::ServerConfig as RustlsServerConfig};

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
        if tls_cfg.enabled {
            // Load certs
            let certs = {
                let file = fs::File::open(tls_cfg.cert_path.as_ref().unwrap())
                    .expect("Cannot open cert file");
                let mut reader = BufReader::new(file);
                rustls_pemfile::certs(&mut reader)
                    .into_iter()
                    .map(|v| v.expect("Failed to read certificate"))
                    .collect::<Vec<_>>()
            };

            let key = {
                let file = fs::File::open(tls_cfg.key_path.as_ref().unwrap())
                    .expect("Cannot open key file");
                let mut reader = BufReader::new(file);
                let first_key = rustls_pemfile::pkcs8_private_keys(&mut reader)
                    .next()
                    .ok_or("No private keys found")
                    .expect("Failed to read private key")
                    .expect("Failed to read private key");

                PrivateKeyDer::Pkcs8(first_key)
            };

            let tls_config = RustlsServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(certs, key)
                .expect("Failed to create TLS config");

            let acceptor = TlsAcceptor::from(Arc::new(tls_config));
            let listener = TcpListener::bind(&server_address).await.unwrap();
            println!("🔒 TLS Server listening on {}", &server_address);

            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let acceptor = acceptor.clone();
                        let sd_clone = sender.clone();

                        tokio::spawn(async move {
                            match acceptor.accept(stream).await {
                                Ok(tls_stream) => {
                                    handle_client(Box::new(tls_stream), sd_clone).await
                                }
                                Err(e) => eprintln!("TLS handshake failed: {:?}", e),
                            }
                        });
                    }
                    Err(e) => eprintln!("Failed to accept connection: {:?}", e),
                }
            }
        } else {
            start_plain_server(&server_address, sender).await;
        }
    } else {
        start_plain_server(&server_address, sender).await;
    }
}

/// Helper to start plain TCP server
async fn start_plain_server(addr: &str, sender: Arc<AsyncMutex<mpsc::UnboundedSender<Packet>>>) {
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("🚀 Plain TCP Server listening on {}", addr);

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
