use clap::Parser;
use lib::types::{RsaPrivateKey, RsaPublicKey};
use tokio::io::{AsyncRead, AsyncWrite};

#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    pub hostname: String,
    pub username: String,
    pub user_id: String,
    pub tls: bool,

    pub public_key_str: String,
    pub public_key: RsaPublicKey,
    pub private_key: RsaPrivateKey,
}

/// A program to run client chat application
#[derive(Parser, Debug)]
#[command(version = "1", about = "Client application for connecting to null-talk server", long_about = None)]
pub struct Args {
    /// Server address, example: localhost:8080
    #[arg(long)]
    pub host: String,

    /// Path to RSA key, example: ~/.ssh/id_rsa
    #[arg(short, long)]
    pub key_path: String,

    /// Whether to use TLS or not
    #[arg(short, long, default_value_t = false)]
    pub tls: bool,
}

pub trait AsyncStream: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> AsyncStream for T {}
