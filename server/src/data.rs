use crate::utils::types::ServerConfig;
use dashmap::DashMap;
use lib::protocol::Packet;
use std::sync::{Arc, OnceLock};
use tokio::sync::mpsc;

#[derive(Clone, Debug)]
pub struct Client {
    pub username: String,
    pub user_id: String,
    pub public_key: String,

    pub session_key: Vec<u8>,
    pub tx: mpsc::Sender<Packet>,
}

pub type PeerMap = Arc<DashMap<String, Client>>;

pub static CONFIG: OnceLock<ServerConfig> = OnceLock::new();
