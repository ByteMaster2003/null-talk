use rsa::{RsaPrivateKey, RsaPublicKey};

#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    pub hostname: String,
    pub username: String,
    pub user_id: String,

    pub public_key_str: String,
    pub public_key: RsaPublicKey,
    pub private_key: RsaPrivateKey,
}
