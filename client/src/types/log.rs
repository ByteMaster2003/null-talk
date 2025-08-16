use std::time::Duration;

use crate::data;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    INFO,
    ERROR,
}

pub struct LogMessage {
    pub level: LogLevel,
    pub msg: String,
    pub hide_after: Duration,
}

impl LogMessage {
    pub async fn log(level: LogLevel, msg: String, hide_after: u64) {
        let log_tx = {
            let channels = data::CHANNELS.lock().await;
            channels.log_tx.clone()
        };

        let _ = log_tx
            .lock()
            .await
            .send(LogMessage {
                level,
                msg,
                hide_after: Duration::from_secs(hide_after),
            })
            .await;
    }
}
