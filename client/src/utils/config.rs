use std::{collections::HashMap, fs};

use crate::{data, types::ConnectionConfig, utils};
use common::utils::{
    enc::{self as encutils},
    file::resolve_path,
    read_file_contents,
};
use config::{Config, File};

fn parse_client_config(path: &str) -> Option<ConnectionConfig> {
    let config_path = match resolve_path(path) {
        Ok(path) => path,
        Err(_) => {
            eprintln!("❗️Invalid file path: {}", path);
            return None;
        }
    };

    let file = File::with_name(config_path.to_str().unwrap());
    let cfg = match Config::builder().add_source(file).build() {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("❗️Failed to parse configuration: {:?}", err);
            return None;
        }
    };

    let config = match cfg.try_deserialize::<HashMap<String, String>>() {
        Ok(map) => map,
        Err(err) => {
            eprintln!("❗️Failed to deserialize configuration: {:?}", err);
            return None;
        }
    };
    let hostname = config.get("hostname").cloned().expect("Missing hostname");
    let port = config.get("port").cloned().expect("Missing port");
    let name = config.get("name").cloned().expect("Missing name");

    let public_key = match resolve_path(
        config
            .get("public_key")
            .cloned()
            .expect("❗️Missing public_key"),
    ) {
        Ok(path) => match fs::read_to_string(&path) {
            Ok(content) => match encutils::parse_public_key(&content) {
                Ok(key) => key,
                Err(_) => {
                    eprintln!("❗️Failed to parse public_key: {:?}", &path);
                    return None;
                }
            },
            Err(_) => {
                eprintln!("❗️Failed to read public_key file: {:?}", &path);
                return None;
            }
        },
        Err(_) => {
            eprintln!("❗️Invalid public_key path");
            return None;
        }
    };

    let private_key = match resolve_path(
        config
            .get("private_key")
            .cloned()
            .expect("❗️Missing private_key"),
    ) {
        Ok(path) => match fs::read_to_string(&path) {
            Ok(content) => match encutils::parse_private_key(&content) {
                Ok(key) => key,
                Err(_) => {
                    eprintln!("❗️Failed to parse private_key: {:?}", &path);
                    return None;
                }
            },
            Err(_) => {
                eprintln!("❗️Failed to read private_key file: {:?}", &path);
                return None;
            }
        },
        Err(_) => {
            eprintln!("❗️Invalid private_key path");
            return None;
        }
    };

    let user_id = encutils::public_key_to_user_id(&public_key);

    Some(ConnectionConfig {
        hostname,
        port,
        name,
        user_id,
        public_key,
        private_key,
    })
}

pub async fn configure_client(args: &[String]) {
    let mut config = data::CLIENT_CONFIG.lock().await;
    if args.len() == 2 {
        let config_path = &args[1];
        match parse_client_config(config_path) {
            Some(cfg) => *config = Some(cfg),
            None => return,
        };
    } else {
        // Ask connection config
        let hostname = utils::take_user_input("Enter server hostname: ");
        let port = utils::take_user_input("Enter port: ");
        let name = utils::take_user_input("Enter username: ");

        let public_key = utils::take_file_input("Enter public key path: ");
        let rsa_public_key = match read_file_contents(&public_key) {
            Ok(contents) => match encutils::parse_public_key(&contents) {
                Ok(key) => key,
                Err(e) => {
                    eprintln!("❌ Failed to parse public key: {}", e);
                    return;
                }
            },
            Err(e) => {
                eprintln!("❌ Failed to read public key file: {}", e);
                return;
            }
        };
        let private_key = utils::take_file_input("Enter private key path: ");
        let rsa_private_key = match read_file_contents(&private_key) {
            Ok(contents) => match encutils::parse_private_key(&contents) {
                Ok(key) => key,
                Err(e) => {
                    eprintln!("❌ Failed to parse private key: {}", e);
                    return;
                }
            },
            Err(e) => {
                eprintln!("❌ Failed to read private key file: {}", e);
                return;
            }
        };

        let user_id = encutils::public_key_to_user_id(&rsa_public_key);

        *config = Some(ConnectionConfig {
            hostname,
            port,
            name,
            user_id,
            public_key: rsa_public_key,
            private_key: rsa_private_key,
        });
    }
}
