use crate::types::LogMessage;
use lib::protocol::Packet;
use tokio::sync::{broadcast, mpsc};

#[derive(Debug)]
pub struct ChannelRegistry {
    pub log_tx: mpsc::Sender<LogMessage>,
    pub pkt_tx: mpsc::Sender<Packet>,
    pub shutdown_tx: broadcast::Sender<bool>,
}

impl ChannelRegistry {
    pub fn new() -> (Self, mpsc::Receiver<LogMessage>, mpsc::Receiver<Packet>) {
        let (log_tx, log_rx) = mpsc::channel::<LogMessage>(10);
        let (pkt_tx, pkt_rx) = mpsc::channel::<Packet>(100);
        let (shutdown_tx, _) = broadcast::channel::<bool>(1);

        (
            Self {
                log_tx,
                pkt_tx,
                shutdown_tx,
            },
            log_rx,
            pkt_rx,
        )
    }
}
