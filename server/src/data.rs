use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

use crate::types::{Client, DmChat, GroupChat};

/**
 * Shared mutable state for the connection configuration.
 */
pub static CLIENTS: LazyLock<Arc<Mutex<HashMap<String, Client>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

pub static CONVERSATIONS: LazyLock<Arc<Mutex<HashMap<String, DmChat>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

pub static GROUPS: LazyLock<Arc<Mutex<HashMap<String, GroupChat>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));
