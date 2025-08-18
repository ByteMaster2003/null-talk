use crate::types::LogMessage;
use std::sync::Arc;
use tokio::sync::{
    Mutex as AsyncMutex,
    mpsc::{Receiver, Sender, channel},
};

/// ### A centralized container for application communication channels.
///
/// [`AppChannels`] groups together asynchronous message-passing channels
/// for logs, commands, and messages. Each channel is created with a bounded
/// capacity of 10 and is wrapped in [`Arc`] + [`tokio::sync::Mutex`] for
/// thread-safe, shared access across tasks.
///
/// # Channels
///
/// - **Logs**
///   - `log_tx`: Sender for [`LogMessage`] events.
///   - `log_rx`: Receiver for [`LogMessage`] events.
/// - **Commands**
///   - `cmd_tx`: Sender for application commands (`String`).
///   - `cmd_rx`: Receiver for application commands.
/// - **Messages**
///   - `msg_tx`: Sender for application messages (`String`).
///   - `msg_rx`: Receiver for application messages.
///
/// # Examples
///
/// ```no_run
/// use crate::types::LogMessage;
/// use tokio::spawn;
///
/// #[tokio::main]
/// async fn main() {
///     let channels = AppChannels::new();
///
///     // Spawn a task that listens for log messages
///     let log_rx = channels.log_rx.clone();
///     spawn(async move {
///         let mut rx = log_rx.lock().await;
///         while let Some(log) = rx.recv().await {
///             println!("LOG: {:?}", log);
///         }
///     });
/// }
/// ```
///
/// [`Arc`]: std::sync::Arc
/// [`LogMessage`]: crate::types::LogMessage
pub struct AppChannels {
    /// Sender for [`LogMessage`] events.
    pub log_tx: Arc<AsyncMutex<Sender<LogMessage>>>,
    /// Receiver for [`LogMessage`] events.
    pub log_rx: Arc<AsyncMutex<Receiver<LogMessage>>>,

    /// Sender for application commands (`String`).
    pub cmd_tx: Arc<AsyncMutex<Sender<String>>>,
    /// Receiver for application commands (`String`).
    pub cmd_rx: Arc<AsyncMutex<Receiver<String>>>,

    /// Sender for application messages (`String`).
    pub msg_tx: Arc<AsyncMutex<Sender<String>>>,
    /// Receiver for application messages (`String`).
    pub msg_rx: Arc<AsyncMutex<Receiver<String>>>,
}

impl AppChannels {
    /// Creates a new [`AppChannels`] instance with all channels initialized.
    ///
    /// Each channel is bounded to a capacity of 10 messages.
    pub fn new() -> Self {
        let (log_tx, log_rx) = channel::<LogMessage>(10);
        let (cmd_tx, cmd_rx) = channel::<String>(10);
        let (msg_tx, msg_rx) = channel::<String>(10);

        AppChannels {
            log_tx: Arc::new(AsyncMutex::new(log_tx)),
            log_rx: Arc::new(AsyncMutex::new(log_rx)),
            cmd_tx: Arc::new(AsyncMutex::new(cmd_tx)),
            cmd_rx: Arc::new(AsyncMutex::new(cmd_rx)),
            msg_tx: Arc::new(AsyncMutex::new(msg_tx)),
            msg_rx: Arc::new(AsyncMutex::new(msg_rx)),
        }
    }
}
