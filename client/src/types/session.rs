use common::types::EncryptionConfig;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

/**
 * Represents the current chat mode (none, direct message, or group).
 */
#[derive(Clone, Debug)]
pub enum ChatMode {
    Dm(String),
    Group(String),
}

/**
 * Represents a user session, including encryption and chat mode.
 */
#[derive(Clone, Debug)]
pub struct Session {
    pub name: String,
    pub encryption: EncryptionConfig,
    pub mode: ChatMode,
    pub id: String,
}

pub struct Message {
    pub sender_pub_key: Option<String>,
    pub receiver_pub_key: Option<String>,
    pub group_id: Option<String>,
    pub content: String,
    pub timestamps: u64,
}

pub type ActiveSession = Arc<Mutex<Option<Session>>>;
pub type Sessions = Arc<Mutex<HashMap<String, Session>>>;
pub type Messages = Arc<Mutex<Vec<Message>>>;
pub type StreamReader = Arc<tokio::sync::Mutex<OwnedReadHalf>>;
pub type StreamWriter = Arc<tokio::sync::Mutex<OwnedWriteHalf>>;
