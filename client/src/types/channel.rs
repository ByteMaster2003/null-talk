use std::sync::Arc;

use tokio::sync::{
    Mutex as AsyncMutex,
    mpsc::{Receiver, Sender, channel},
};

use crate::types::LogMessage;

#[derive(Debug)]
pub struct AppChannels {
    pub log_tx: Arc<AsyncMutex<Sender<LogMessage>>>,
    pub log_rx: Arc<AsyncMutex<Receiver<LogMessage>>>,

    pub cmd_tx: Arc<AsyncMutex<Sender<String>>>,
    pub cmd_rx: Arc<AsyncMutex<Receiver<String>>>,

    pub msg_tx: Arc<AsyncMutex<Sender<String>>>,
    pub msg_rx: Arc<AsyncMutex<Receiver<String>>>,
}

impl AppChannels {
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
