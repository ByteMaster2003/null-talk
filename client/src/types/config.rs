use rsa::{RsaPrivateKey, RsaPublicKey};
use serde::Deserialize;

/// ### Represents the configuration for a connection.
/// 
/// This struct holds the necessary details for connecting to a server,
#[derive(Debug, Deserialize, Clone)]
pub struct ConnectionConfig {
    pub hostname: String,
    pub port: String,
    pub name: String,
    pub user_id: String,

    pub public_key: RsaPublicKey,
    pub private_key: RsaPrivateKey,
}
