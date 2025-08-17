use crate::{
    handlers::task,
    types::{LogLevel, LogMessage},
    utils::perform_handshake,
};
use std::sync::Arc;
use tokio::{net::TcpStream, sync::Mutex};

pub async fn handle_client(stream: TcpStream) {
    let (rd, wt) = stream.into_split();
    let rd = Arc::new(Mutex::new(rd));
    let wt = Arc::new(Mutex::new(wt));

    let _ = match perform_handshake(rd.clone(), wt.clone()).await {
        Ok(session_key) => session_key,
        Err(e) => {
            let _ = LogMessage::log(LogLevel::ERROR, format!("Handshake failed: {}", e), 0).await;
            return;
        }
    };

    // 2. Message Writer Transmitter Task
    let wt_task = task::start_writer_task(wt.clone()).await;

    // 1. Task to receive incoming messages
    let rd_task = task::start_reader_task(rd.clone()).await;

    // 3. Command Handler Task
    let cmd_task = task::start_command_task(rd.clone(), wt.clone(), rd_task).await;

    let _ = cmd_task.await;
    wt_task.abort();
}
