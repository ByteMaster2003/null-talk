use std::{collections::HashMap, fs};

use crate::types::ConnectionConfig;
use common::utils::{
    enc::{parse_private_key, parse_public_key},
    file::resolve_path,
};
use config::{Config, File};

pub fn parse_client_config(path: &str) -> Option<ConnectionConfig> {
    let config_path = match resolve_path(path) {
        Ok(path) => path,
        Err(_) => {
            eprintln!("❗️MInvalid file path: {}", path);
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
            Ok(content) => match parse_public_key(&content) {
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
            Ok(content) => match parse_private_key(&content) {
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

    Some(ConnectionConfig {
        hostname,
        port,
        name,
        public_key,
        private_key,
    })
}
