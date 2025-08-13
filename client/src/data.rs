use crate::types::{ActiveSession, ConnectionConfig, Messages, Sessions};
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

/**
 * Shared mutable state for the connection configuration.
 */
pub static CLIENT_CONFIG: LazyLock<Arc<Mutex<Option<ConnectionConfig>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(None)));

/**
 * Shared mutable state for all active IRC sessions.
 */
pub static SESSIONS: LazyLock<Sessions> = LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

/**
 * Shared mutable state for the active IRC session.
 */
pub static ACTIVE_SESSION: LazyLock<ActiveSession> = LazyLock::new(|| Arc::new(Mutex::new(None)));

/**
 * Shared mutable state for all messages.
 */
pub static MESSAGES: LazyLock<Arc<HashMap<String, Messages>>> =
    LazyLock::new(|| Arc::new(HashMap::new()));
