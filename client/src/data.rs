use crate::utils::types::ConnectionConfig;
use std::sync::OnceLock;

pub static CLIENT: OnceLock<ConnectionConfig> = OnceLock::new();
