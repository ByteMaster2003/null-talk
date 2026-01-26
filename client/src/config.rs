use crate::{
    data,
    utils::types::{Args, ConnectionConfig},
};
use lib::crypto;
use std::path::Path;

pub fn config(args: Args) {
    let base_path = args.key_path;
    let priv_path = Path::new(&base_path);
    let username = priv_path.file_name().unwrap().to_string_lossy().to_string();
    let pub_path = format!("{}.pub", &base_path);
    let pub_path = Path::new(&pub_path);
    let hostname = args.host;

    // 1. Load Keys (Handles passphrase prompt automatically)
    let private_key = match crypto::parse_private_key(priv_path) {
        Ok(k) => k,
        Err(e) => {
            panic!("Error: {}", e.to_string());
        }
    };

    let (public_key, public_key_str) = match crypto::parse_public_key(pub_path) {
        Ok(r) => r,
        Err(e) => {
            panic!("Error: {}", e.to_string());
        }
    };

    let user_id = crypto::public_key_to_user_id(&public_key);

    data::CLIENT
        .set(ConnectionConfig {
            username,
            user_id,
            hostname: hostname.clone(),
            private_key,
            public_key,
            public_key_str,
            tls: args.tls,
        })
        .ok()
        .expect("Failed to save config");
}
