use clap::Parser;
use tokio::io::{AsyncRead, AsyncWrite};

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: String,

    pub tls: bool,
    pub domain: Option<String>,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
}

/// A program to run client chat application
#[derive(Parser, Debug)]
#[command(version = "1", about = "null-talk server", long_about = None)]
pub struct Args {
    /// TCP Port example: 8080
    #[arg(short, long, default_value_t = String::from("8080"))]
    pub port: String,

    /// Domain Name, example: null.talk.com
    #[arg(short, long)]
    pub domain: Option<String>,
}

pub trait AsyncStream: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> AsyncStream for T {}
