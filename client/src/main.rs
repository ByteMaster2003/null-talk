use null_talk_client::{
    data,
    handlers::handle_client,
    types::{LogLevel, LogMessage},
    ui::run_terminal,
    utils::configure_client,
};
use std::env;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if !configure_client(&args).await {
        return;
    }

    // Create TCP connection thread
    let tcp = tokio::spawn(async move {
        let addr = {
            let config_lock = data::CLIENT_CONFIG.lock().await;
            let config = match config_lock.as_ref() {
                Some(cfg) => cfg,
                None => {
                    LogMessage::log(LogLevel::ERROR, "Configuration not found!".into(), 0).await;
                    return;
                }
            };

            format!("{}:{}", &config.hostname, &config.port)
        };

        match TcpStream::connect(addr.clone()).await {
            Ok(stream) => {
                LogMessage::log(
                    LogLevel::INFO,
                    format!("Successfully connected to {}", addr.clone()),
                    5,
                )
                .await;

                handle_client(stream).await;
            }
            Err(_) => {
                LogMessage::log(LogLevel::ERROR, format!("Failed to connect to {}", addr), 0).await;
                return;
            }
        }
    });

    color_eyre::install().expect("Failed to install color_eyre");
    let terminal = ratatui::init();

    let _ = run_terminal(terminal)
        .await
        .expect("Failed to run terminal");

    ratatui::restore();
    tcp.abort();
}
