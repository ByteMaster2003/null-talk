use std::{collections::HashMap, path::PathBuf};

use config::{Config, File};

use crate::{
    data::SESSIONS,
    types::{LogLevel, LogMessage, Session},
};
use common::{
    net::{ChatMessageKind, Packet, StreamReader, StreamWriter},
    types::{
        ChatMode, EncryptionConfig, NewSessionPayload, NewSessionResponse, ServerResponse,
        SymmetricAlgo,
    },
    utils::{file::resolve_path, net as netutils},
};

pub async fn new_connection(input: &str, rd: StreamReader, wt: StreamWriter) -> Option<Session> {
    let path = match resolve_path(input) {
        Ok(path) => path,
        Err(_) => {
            let _ =
                LogMessage::log(LogLevel::ERROR, format!("Invalid file path: {}", input), 5).await;
            return None;
        }
    };

    let mut session = match parse_connection_file(&path).await {
        Some(session) => session,
        None => {
            let _ = LogMessage::log(
                LogLevel::ERROR,
                format!("Failed to parse connection file: {:?}", path),
                5,
            )
            .await;
            return None;
        }
    };

    let new_session_payload = NewSessionPayload {
        id: session.id.clone(),
        mode: session.mode.clone(),
        algo: session.encryption.algo.clone(),
    };

    let packet = Packet {
        kind: ChatMessageKind::Command("new".to_string()),
        payload: match bincode::encode_to_vec(&new_session_payload, bincode::config::standard()) {
            Ok(vec) => vec,
            Err(e) => {
                let _ = LogMessage::log(LogLevel::ERROR, format!("Something went wrong: {}", e), 5)
                    .await;
                return None;
            }
        },
    };

    if let Err(_) = netutils::write_packet(wt.clone(), packet).await {
        let _ = LogMessage::log(
            LogLevel::ERROR,
            format!("Something went wrong, pls check your network connection"),
            5,
        )
        .await;
        return None;
    }

    let response: ServerResponse = match netutils::read_packet(rd.clone()).await {
        Ok(packet) => packet,
        Err(err) => {
            let _ = LogMessage::log(
                LogLevel::ERROR,
                format!("Failed to read response packet: {}", err),
                5,
            )
            .await;
            return None;
        }
    };

    if !response.success {
        let _ = LogMessage::log(
            LogLevel::ERROR,
            format!(
                "{}",
                response
                    .error
                    .unwrap_or_else(|| "Failed to create new session".into())
            ),
            5,
        )
        .await;

        return None;
    }

    let payload = match response.payload {
        Some(ref payload) => payload,
        None => {
            let _ = LogMessage::log(LogLevel::ERROR, format!("Something went wrong!"), 5).await;
            return None;
        }
    };
    let (new_session, _): (NewSessionResponse, usize) =
        match bincode::decode_from_slice(&payload, bincode::config::standard()) {
            Ok(chat) => chat,
            Err(err) => {
                let _ =
                    LogMessage::log(LogLevel::ERROR, format!("Something went wrong: {}", err), 5)
                        .await;
                return None;
            }
        };

    session.id = new_session.id;
    session.encryption.encryption_key = Some(new_session.session_key);

    Some(session)
}

pub async fn get_session(key: &str) -> Option<Session> {
    let sessions = SESSIONS.lock().await;
    sessions.get(key).cloned()
}

pub async fn rm_connection(key: &str) {
    let mut sessions = SESSIONS.lock().await;

    match sessions.remove(key) {
        Some(session) => {
            let _ = LogMessage::log(
                LogLevel::INFO,
                format!("Removed connection: {:?} {:?}", session.mode, key),
                5,
            )
            .await;
            session
        }
        None => {
            let _ = LogMessage::log(
                LogLevel::ERROR,
                format!("No connection found for key: {}", key),
                5,
            )
            .await;
            return;
        }
    };
}

async fn parse_connection_file(path: &PathBuf) -> Option<Session> {
    let file = File::with_name(path.to_str().unwrap());
    let cfg = Config::builder().add_source(file).build().unwrap();

    let deserialized = match cfg.try_deserialize::<HashMap<String, String>>() {
        Ok(map) => map,
        Err(_) => return None,
    };

    let name = match deserialized.get("name").cloned() {
        Some(n) => n,
        None => {
            let _ = LogMessage::log(LogLevel::ERROR, format!("Name is required!"), 5).await;
            return None;
        }
    };

    let mode = match deserialized.get("connection_type").cloned() {
        Some(t) => match t.as_str() {
            "dm" => ChatMode::Dm(name.clone()),
            "group" => ChatMode::Group(name.clone()),
            _ => {
                let _ =
                    LogMessage::log(LogLevel::ERROR, format!("Unknown connection type"), 5).await;
                return None;
            }
        },
        None => {
            let _ =
                LogMessage::log(LogLevel::ERROR, format!("Connection type is required!"), 5).await;
            return None;
        }
    };

    let id = match deserialized.get("id").cloned() {
        Some(id) => id,
        None => {
            let _ =
                LogMessage::log(LogLevel::ERROR, format!("Missing 'id' in configuration"), 5).await;
            return None;
        }
    };

    let encryption = match get_enc_config(&deserialized).await {
        Some(config) => config,
        None => {
            let _ = LogMessage::log(
                LogLevel::ERROR,
                format!("Failed to get encryption config"),
                5,
            )
            .await;
            return None;
        }
    };

    let new_session = Session {
        id,
        mode,
        name,
        encryption,
    };

    Some(new_session)
}

async fn get_enc_config(deserialized: &HashMap<String, String>) -> Option<EncryptionConfig> {
    let algo = match deserialized.get("algo").cloned() {
        Some(method) => match method.as_str() {
            "AES256" => SymmetricAlgo::AES256,
            "ChaCha20" => SymmetricAlgo::ChaCha20,
            _ => {
                let _ = LogMessage::log(
                    LogLevel::ERROR,
                    format!("Invalid algo, supported values are: AES256, ChaCha20"),
                    5,
                )
                .await;
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
