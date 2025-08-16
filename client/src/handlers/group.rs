use std::path::PathBuf;

use config::{Config, File};

use crate::types::{LogLevel, LogMessage, Session};
use common::{
    net::{ChatMessageKind, Packet, StreamReader, StreamWriter},
    types::{
        ChatMode, EncryptionConfig, NewGroupPayload, NewGroupResponse, ServerResponse,
        SymmetricAlgo,
    },
    utils::{file::resolve_path, net as netutils},
};

pub async fn create_new_group(
    file_path: &str,
    rd: StreamReader,
    wt: StreamWriter,
) -> Option<Session> {
    let path = match resolve_path(file_path) {
        Ok(path) => path,
        Err(_) => {
            let _ = LogMessage::log(
                LogLevel::ERROR,
                format!("Invalid file path: {:?}", file_path),
                5,
            )
            .await;
            return None;
        }
    };

    let payload = match parse_group_file(&path) {
        Some(info) => info,
        None => {
            let _ = LogMessage::log(
                LogLevel::ERROR,
                format!("Failed to parse group file: {:?}", path),
                5,
            )
            .await;
            return None;
        }
    };

    let packet = Packet {
        kind: ChatMessageKind::Command("mkgp".to_string()),
        payload: match bincode::encode_to_vec(&payload, bincode::config::standard()) {
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
        Ok(resp) => resp,
        Err(e) => {
            let _ = LogMessage::log(
                LogLevel::ERROR,
                format!("Failed to read response: {}", e),
                5,
            )
            .await;
            return None;
        }
    };

    if !response.success {
        let msg = response.error.clone();
        let _ = LogMessage::log(
            LogLevel::ERROR,
            format!(
                "Failed to create group: {}",
                msg.unwrap_or_else(|| "".to_string())
            ),
            5,
        )
        .await;
        return None;
    }
    let group_info = match response.payload {
        Some(payload) => {
            let (group_info, _): (NewGroupResponse, usize) =
                match bincode::decode_from_slice(&payload, bincode::config::standard()) {
                    Ok(info) => info,
                    Err(_) => {
                        let _ = LogMessage::log(
                            LogLevel::ERROR,
                            format!("Failed to decode group info"),
                            5,
                        )
                        .await;
                        return None;
                    }
                };

            group_info
        }
        None => {
            let _ = LogMessage::log(LogLevel::ERROR, format!("Failed to create group"), 5).await;
            return None;
        }
    };

    Some(Session {
        name: payload.name.clone(),
        id: group_info.group_id,
        mode: ChatMode::Group(payload.name.clone()),
        encryption: EncryptionConfig {
            algo: SymmetricAlgo::AES256,
            encryption_key: Some(group_info.session_key),
        },
    })
}

pub async fn add_group_member(member_id: &str, rd: StreamReader, wt: StreamWriter) {
    let packet = Packet {
        kind: ChatMessageKind::Command("addgpm".to_string()),
        payload: match bincode::encode_to_vec(member_id, bincode::config::standard()) {
            Ok(vec) => vec,
            Err(e) => {
                let _ = LogMessage::log(LogLevel::ERROR, format!("Something went wrong: {}", e), 5)
                    .await;
                return;
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

        return;
    }

    let response: ServerResponse = match netutils::read_packet(rd.clone()).await {
        Ok(resp) => resp,
        Err(e) => {
            let _ = LogMessage::log(
                LogLevel::ERROR,
                format!("Failed to read response: {}", e),
                5,
            )
            .await;
            return;
        }
    };

    if !response.success {
        let msg = response.error.clone();
        let _ = LogMessage::log(
            LogLevel::ERROR,
            format!(
                "Failed to add group member: {}",
                msg.unwrap_or_else(|| "".to_string())
            ),
            5,
        )
        .await;
        return;
    } else {
        if let Some(payload) = response.payload {
            let (msg, _): (String, usize) =
                match bincode::decode_from_slice(&payload, bincode::config::standard()) {
                    Ok(data) => data,
                    Err(err) => {
                        let _ = LogMessage::log(
                            LogLevel::ERROR,
                            format!("Something went wrong: {}", err),
                            5,
                        )
                        .await;
                        return;
                    }
                };

            let _ = LogMessage::log(LogLevel::INFO, format!("{}", msg), 5).await;
            return;
        }
        let _ = LogMessage::log(
            LogLevel::INFO,
            format!("Group member added successfully"),
            5,
        )
        .await;
    }
}

pub fn parse_group_file(path: &PathBuf) -> Option<NewGroupPayload> {
    let file = File::with_name(path.to_str().unwrap());
    let cfg = Config::builder().add_source(file).build().unwrap();

    let group_info = match cfg.try_deserialize::<NewGroupPayload>() {
        Ok(group) => group,
        Err(_) => return None,
    };

    Some(group_info)
}
