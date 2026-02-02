use chrono::Local;

use crate::data;
use std::{fs, io::Write, time::Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    INFO,
    ERROR,
}
pub struct LogMessage {
    /// The severity level of the log message.
    pub level: LogLevel,
    /// The actual log message text.
    pub msg: String,
    /// How long the message should be displayed before being hidden.
    pub hide_after: Duration,
}

impl LogMessage {
    pub async fn log(level: LogLevel, msg: String, hide_after: u64) {
        let log_tx = {
            let channels = data::CHANNELS.get().unwrap();
            channels.log_tx.clone()
        };

        let _ = log_tx
            .send(LogMessage {
                level,
                msg,
                hide_after: Duration::from_secs(hide_after),
            })
            .await;
    }

    pub fn log_to_file(message: String) -> std::io::Result<()> {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("app.log")?;

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        writeln!(file, "[{}] {}", timestamp, message)?;
        Ok(())
    }
}
