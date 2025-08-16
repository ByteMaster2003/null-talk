use common::types::{ChatMode, EncryptionConfig, Message};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex as AsyncMutex;

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

pub type ActiveSession = Arc<AsyncMutex<Option<Session>>>;
pub type Sessions = Arc<AsyncMutex<HashMap<String, Session>>>;
pub type Messages = Arc<AsyncMutex<Vec<Message>>>;
