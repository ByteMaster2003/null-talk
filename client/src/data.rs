use tokio::sync::Mutex as AsyncMutex;

use crate::types::{ActiveSession, AppChannels, AppConfig, ConnectionConfig, Messages, Sessions};
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

/**
 * Shared mutable state for the connection configuration.
 */
pub static CLIENT_CONFIG: LazyLock<Arc<AsyncMutex<Option<ConnectionConfig>>>> =
    LazyLock::new(|| Arc::new(AsyncMutex::new(None)));

/**
 * Shared mutable state for all active IRC sessions.
 */
pub static SESSIONS: LazyLock<Sessions> =
    LazyLock::new(|| Arc::new(AsyncMutex::new(HashMap::new())));

/**
 * Shared mutable state for the active IRC session.
 */
pub static ACTIVE_SESSION: LazyLock<ActiveSession> =
    LazyLock::new(|| Arc::new(AsyncMutex::new(None)));

/**
 * Shared mutable state for all messages.
 */
pub static MESSAGES: LazyLock<Arc<AsyncMutex<HashMap<String, Messages>>>> =
    LazyLock::new(|| Arc::new(AsyncMutex::new(HashMap::new())));

/**
 * Shared mutable terminal state.
 */
pub static APP_STATE: LazyLock<Arc<Mutex<AppConfig>>> =
    LazyLock::new(|| Arc::new(Mutex::new(AppConfig::new())));

/**
 * Shared mutable state for application channels.
 */
pub static CHANNELS: LazyLock<Arc<AsyncMutex<AppChannels>>> =
    LazyLock::new(|| Arc::new(AsyncMutex::new(AppChannels::new())));
