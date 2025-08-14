use std::{
    collections::{HashMap, hash_map::Entry},
    path::PathBuf,
};

use config::{Config, File};

use crate::{data::SESSIONS, types::Session};
use common::{
    net::{ChatMessageKind, Packet, StreamReader, StreamWriter},
    types::{
        ChatMode, EncryptionConfig, NewSessionPayload, NewSessionResponse, ServerResponse,
        SymmetricAlgo,
    },
    utils::{enc::hash_string, file::resolve_path, net as netutils},
};

pub async fn new_connection(input: &str, rd: StreamReader, wt: StreamWriter) -> Option<Session> {
    let path = match resolve_path(input) {
        Ok(path) => path,
        Err(_) => {
            eprintln!("Invalid file path: {}", input);
            return None;
        }
    };

    let mut session = match parse_connection_file(&path) {
        Some(session) => session,
        None => {
            eprintln!("Failed to parse connection file: {:?}", path);
            return None;
        }
    };

    let new_session_payload = NewSessionPayload {
        id: session.id.clone(),
        mode: session.mode.clone(),
        algo: session.encryption.algo.clone(),
    };

    let packet = Packet {
        kind: ChatMessageKind::Command("/new".to_string()),
        payload: bincode::encode_to_vec(&new_session_payload, bincode::config::standard())
            .unwrap_or("Failed to encode payload".into()),
    };

    if let Err(_) = netutils::write_packet(wt.clone(), packet).await {
        eprintln!("Failed to send packet");
        return None;
    }

    let response: ServerResponse = match netutils::read_packet(rd.clone()).await {
        Ok(packet) => packet,
        Err(err) => {
            eprintln!("Failed to read response packet: {}", err);
            return None;
        }
    };

    if !response.success {
        eprintln!(
            "Error: {}",
            response
                .error
                .unwrap_or_else(|| "Failed to create new session".into())
        );
        return None;
    }

    let payload = match response.payload {
        Some(ref payload) => payload,
        None => {
            eprintln!("No payload found in response");
            return None;
        }
    };
    let (new_session, _): (NewSessionResponse, usize) =
        match bincode::decode_from_slice(&payload, bincode::config::standard()) {
            Ok(chat) => chat,
            Err(err) => {
                eprintln!("Error: {}", err);
                return None;
            }
        };

    session.id = new_session.id;
    session.encryption.encryption_key = Some(new_session.session_key);

    Some(session)
}

pub fn get_session(key: &str) -> Option<Session> {
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
        None => SymmetricAlgo::AES256,
    };

    Some(EncryptionConfig {
        algo,
        encryption_key: None,
    })
}
