use clap::Parser;
use client::{
    config, data,
    net::{handle_client, tcp::open_tcp_stream},
    types::ChannelRegistry,
    ui,
    utils::types::Args,
};

#[tokio::main]
async fn main() {
    // Parse and save CLI config
    let args = Args::parse();
    config(args);

    // Initialize Channels
    let (channels, log_rx, pkt_rx) = ChannelRegistry::new();
    data::CHANNELS.set(channels).unwrap();

    // Open the TCP connection
    let stream = match open_tcp_stream().await {
        Ok(st) => st,
        Err(e) => panic!("Error: {}", e.to_string()),
    };
    let tcp_task = tokio::spawn(async move { handle_client(stream, pkt_rx).await });

    // Install color_eyre and run UI
    color_eyre::install().expect("Failed to run UI");
    ui::run(ratatui::init(), log_rx)
        .await
        .expect("Failed to run UI");

    tcp_task.abort();
    ratatui::restore();
}
