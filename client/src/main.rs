use clap::Parser;
use client::{
    config,
    net::{handle_client, tcp::open_tcp_stream},
    utils::types::Args,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse and save CLI config
    let args = Args::parse();
    config(args);

    // Open the TCP connection
    let stream = match open_tcp_stream().await {
        Ok(st) => st,
        Err(e) => panic!("Error: {}", e.to_string()),
    };

    let connection = tokio::spawn(async move { handle_client(stream).await });

    let _ = connection.await;
    Ok(())
}
