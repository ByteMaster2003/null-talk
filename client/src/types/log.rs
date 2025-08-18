use crate::data;
use std::time::Duration;

/// ### The severity level of a log message.
///
/// Used in [`LogMessage`] to categorize messages.
///
/// # Variants
/// - [`LogLevel::INFO`] — General informational messages.
/// - [`LogLevel::ERROR`] — Error messages indicating failures or issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// General informational message.
    INFO,
    /// Error message indicating a failure or issue.
    ERROR,
}

/// ### A structured log entry used by the application.
///
/// Each log message has a severity [`LogLevel`], a textual message, and an optional
/// display duration (`hide_after`) to control how long the message should remain visible.
pub struct LogMessage {
    /// The severity level of the log message.
    pub level: LogLevel,
    /// The actual log message text.
    pub msg: String,
    /// How long the message should be displayed before being hidden.
    pub hide_after: Duration,
}

impl LogMessage {
    /// Sends a new log message into the global application log channel.
    ///
    /// This helper acquires a lock on [`data::CHANNELS`], clones the
    /// [`log_tx`](crate::types::AppChannels::log_tx) sender, and pushes a new
    /// [`LogMessage`] onto it.
    ///
    /// # Parameters
    ///
    /// - `level`: The severity level of the log (e.g. [`LogLevel::INFO`], [`LogLevel::ERROR`]).
    /// - `msg`: The log message text.
    /// - `hide_after`: Number of seconds the log should remain visible before being hidden.
    ///
    /// # Errors
    ///
    /// This function ignores errors silently. If the receiver has been dropped,
    /// the message will not be delivered.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use my_crate::{LogMessage, LogLevel};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     LogMessage::log(LogLevel::INFO, "Server started".into(), 5).await;
    ///     LogMessage::log(LogLevel::ERROR, "Database connection failed".into(), 10).await;
    /// }
    /// ```
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
