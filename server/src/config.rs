use crate::{
    data,
    utils::types::{Args, ServerConfig},
};
use std::path::Path;

pub fn config(args: Args) {
    let port = args.port;
    let mut config = ServerConfig {
        port: port.clone(),
        tls: false,
        domain: None,
        cert_path: None,
        key_path: None,
    };

    if let Some(domain) = args.domain {
        let cert_path_str = format!("/etc/letsencrypt/live/{}/fullchain.pem", &domain);
        let cert_path = Path::new(&cert_path_str);
        if !cert_path.exists() {
            panic!("Error: No fullchain.pem found for domain: {}", &domain)
        }

        let key_path_str = format!("/etc/letsencrypt/live/{}/privkey.pem", &domain);
        let key_path = Path::new(&key_path_str);
        if !key_path.exists() {
            panic!("Error: No privkey.pem found for domain: {}", &domain)
        }

        config.domain = Some(domain);
        config.cert_path = Some(cert_path_str);
        config.key_path = Some(key_path_str);
        config.tls = true;
    }

    data::CONFIG
        .set(config)
        .ok()
        .expect("Failed to save config");
}
