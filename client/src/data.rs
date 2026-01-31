use crate::{
    types::{AppState, ChannelRegistry, Message, Session},
    utils::types::ConnectionConfig,
};
use dashmap::DashMap;
use std::sync::{LazyLock, Mutex, OnceLock};

pub static CLIENT: OnceLock<ConnectionConfig> = OnceLock::new();

pub static SESSION_KEY: OnceLock<Vec<u8>> = OnceLock::new();

/// Shared mutable state for all application channels
pub static CHANNELS: OnceLock<ChannelRegistry> = OnceLock::new();

/// Shared UI state
pub static APP_STATE: LazyLock<Mutex<AppState>> = LazyLock::new(|| Mutex::new(AppState::new()));

/// Shared UI state
pub static SESSIONS: LazyLock<DashMap<String, Session>> = LazyLock::new(|| DashMap::new());

/// Shared UI state
pub static MESSAGES: LazyLock<DashMap<String, Vec<Message>>> = LazyLock::new(|| DashMap::new());
