use crate::types::{Client, DmChat, GroupChat};
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};
use tokio::sync::Mutex as AsyncMutex;

/**
 * Shared mutable state for the connection configuration.
 */
pub static CLIENTS: LazyLock<Arc<AsyncMutex<HashMap<String, Client>>>> =
    LazyLock::new(|| Arc::new(AsyncMutex::new(HashMap::new())));

pub static CONVERSATIONS: LazyLock<Arc<AsyncMutex<HashMap<String, DmChat>>>> =
    LazyLock::new(|| Arc::new(AsyncMutex::new(HashMap::new())));

pub static GROUPS: LazyLock<Arc<AsyncMutex<HashMap<String, GroupChat>>>> =
    LazyLock::new(|| Arc::new(AsyncMutex::new(HashMap::new())));
