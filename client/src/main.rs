use client::handlers::handle_client;
use lib::{crypto, types::ConnectionConfig};
use std::path::Path;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: cli-tool <host:port> <key_base_path>");
        return Ok(());
    }

    let base_path = args[2].to_string();
    let priv_path = Path::new(&base_path);
    let username = priv_path.file_name().unwrap().to_string_lossy().to_string();
    let pub_path = format!("{}.pub", &base_path);
    let pub_path = Path::new(&pub_path);
    let hostname = args[1].to_string();

    // 1. Load Keys (Handles passphrase prompt automatically)
    let private_key = crypto::parse_private_key(priv_path)?;
    let (public_key, public_key_str) = crypto::parse_public_key(pub_path)?; // Uses function from previous response
    let user_id = crypto::public_key_to_user_id(&public_key);

    let config = ConnectionConfig {
        username,
        user_id,
        hostname: hostname.clone(),
        private_key,
        public_key,
        public_key_str,
    };

    match TcpStream::connect(hostname.clone()).await {
        Ok(stream) => {
            handle_client(stream, &config).await;
        }
        Err(_) => {
            println!("Failed to connect to {}", hostname.clone())
        }
    };

    Ok(())
}
