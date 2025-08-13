use std::{
    collections::{HashMap, hash_map::Entry},
    path::PathBuf,
};

use config::{Config, File};

use crate::{
    data::SESSIONS,
    types::{ChatMode, Session},
};
use common::{
    types::{EncryptionConfig, SymmetricAlgo},
    utils::{enc::hash_string, file::resolve_path},
};

pub fn new_connection(input: &str) -> Option<Session> {
    let path = match resolve_path(input) {
        Ok(path) => path,
        Err(_) => {
            eprintln!("Invalid file path: {}", input);
            return None;
        }
    };

    match parse_connection_file(&path) {
        Some(session) => Some(session),
        None => {
            eprintln!("Failed to parse connection file: {:?}", path);
            None
        }
    }
}

pub fn get_connection(key: &str) -> Option<Session> {
    let sessions = SESSIONS.lock().unwrap();
    sessions.get(key).cloned()
}

pub fn list_connections() {
    let sessions = SESSIONS.lock().unwrap();
    println!();
    println!(
        "–––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––"
    );
    for (key, session) in sessions.iter() {
        println!("| Connection: {} {:?}", key, session.mode);
    }
    println!(
        "–––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––"
    );
    println!();
}

pub fn rm_connection(key: &str) {
    let mut sessions = SESSIONS.lock().unwrap();

    match sessions.remove(key) {
        Some(session) => {
            println!("Removed connection: {:?} {:?}", session.mode, key);
            session
        }
        None => {
            eprintln!("No connection found for key: {}", key);
            return;
        }
    };
}

fn parse_connection_file(path: &PathBuf) -> Option<Session> {
    let file = File::with_name(path.to_str().unwrap());
    let cfg = Config::builder().add_source(file).build().unwrap();

    let deserialized = match cfg.try_deserialize::<HashMap<String, String>>() {
        Ok(map) => map,
        Err(_) => return None,
    };

    let name = match deserialized.get("name").cloned() {
        Some(n) => n,
        None => {
            eprintln!("Name is required!");
            return None;
        }
    };

    let mode = match deserialized.get("connection_type").cloned() {
        Some(t) => match t.as_str() {
            "dm" => ChatMode::Dm(name.clone()),
            "group" => ChatMode::Group(name.clone()),
            _ => {
                eprintln!("Unknown connection type");
                return None;
            }
        },
        None => {
            eprintln!("Connection type is required!");
            return None;
        }
    };

    let id = match deserialized.get("id").cloned() {
        Some(id) => id,
        None => {
            eprintln!("ID is required!");
            return None;
        }
    };

    let encryption = match get_enc_config(&deserialized) {
        Some(config) => config,
        None => {
            eprintln!("Failed to get encryption config");
            return None;
        }
    };

    let str = format!("{:?}{:?}{:?}", mode, encryption, id.clone());
    let key = hash_string(&str);
    let mut sessions = SESSIONS.lock().unwrap();

    let new_session = Session {
        id,
        mode,
        name,
        encryption,
    };

    match sessions.entry(key.clone()) {
        Entry::Occupied(entry) => Some(entry.get().clone()),
        Entry::Vacant(entry) => {
            entry.insert(new_session.clone());
            sessions.get(&key).cloned()
        }
    };

    Some(new_session)
}

fn get_enc_config(deserialized: &HashMap<String, String>) -> Option<EncryptionConfig> {
    let algo = match deserialized.get("algo").cloned() {
        Some(method) => match method.as_str() {
            "AES256" => SymmetricAlgo::AES256,
            "ChaCha20" => SymmetricAlgo::ChaCha20,
            _ => {
                eprintln!("Invalid algo, supported values are: AES256, ChaCha20");
                return None;
            }
        },
        None => {
            eprintln!("Missing algo");
            return None;
        }
    };

    Some(EncryptionConfig {
        algo,
        encryption_key: None,
    })
}
