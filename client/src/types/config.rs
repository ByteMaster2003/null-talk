use rsa::{RsaPrivateKey, RsaPublicKey};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ConnectionConfig {
    pub hostname: String,
    pub port: String,
    pub name: String,
    pub user_id: String,

    pub public_key: RsaPublicKey,
    pub private_key: RsaPrivateKey,
}
