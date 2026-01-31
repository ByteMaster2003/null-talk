use crate::utils::types::ServerConfig;
use dashmap::DashMap;
use lib::protocol::Packet;
use std::sync::{Arc, LazyLock, OnceLock};
use tokio::sync::mpsc;

#[derive(Clone, Debug)]
pub struct Client {
    pub username: String,
    pub user_id: String,
    pub public_key: String,

    pub session_key: Vec<u8>,
    pub tx: mpsc::Sender<Packet>,
}

pub static CLIENTS: LazyLock<Arc<DashMap<String, Client>>> =
    LazyLock::new(|| Arc::new(DashMap::new()));

pub static CONFIG: OnceLock<ServerConfig> = OnceLock::new();
