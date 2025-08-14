use std::path::PathBuf;

use config::{Config, File};

use crate::types::Session;
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
            eprintln!("Invalid file path: {:?}", file_path);
            return None;
        }
    };

    let payload = match parse_group_file(&path) {
        Some(info) => info,
        None => {
            eprintln!("Failed to parse group file: {:?}", path);
            return None;
        }
    };

    let packet = Packet {
        kind: ChatMessageKind::Command("/mkgp".to_string()),
        payload: bincode::encode_to_vec(&payload, bincode::config::standard())
            .unwrap_or("Failed to encode payload".into()),
    };

    if let Err(_) = netutils::write_packet(wt.clone(), packet).await {
        eprintln!("Failed to send packet");
        return None;
    }

    let response: ServerResponse = match netutils::read_packet(rd.clone()).await {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Failed to read response: {}", e);
            return None;
        }
    };

    if !response.success {
        let msg = response.error.clone();
        eprintln!(
            "Failed to create group: {}",
            msg.unwrap_or_else(|| "".to_string())
        );
        return None;
    }
    let group_info = match response.payload {
        Some(payload) => {
            let (group_info, _): (NewGroupResponse, usize) =
                match bincode::decode_from_slice(&payload, bincode::config::standard()) {
                    Ok(info) => info,
                    Err(_) => {
                        eprintln!("Failed to decode group info");
                        return None;
                    }
                };

            group_info
        }
        None => {
            eprintln!("Failed to create group");
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
        kind: common::net::ChatMessageKind::Command("/addgpm".to_string()),
        payload: bincode::encode_to_vec(member_id, bincode::config::standard())
            .unwrap_or("Failed to encode payload".into()),
    };

    if let Err(_) = netutils::write_packet(wt.clone(), packet).await {
        eprintln!("Failed to send packet");
        return;
    }

    let response: ServerResponse = match netutils::read_packet(rd.clone()).await {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Failed to read response: {}", e);
            return;
        }
    };

    if !response.success {
        let msg = response.error.clone();
        eprintln!("{}", msg.unwrap_or_else(|| "".to_string()));
        return;
    } else {
        if let Some(payload) = response.payload {
            let (msg, _): (String, usize) =
                match bincode::decode_from_slice(&payload, bincode::config::standard()) {
                    Ok(data) => data,
                    Err(err) => {
                        eprintln!("Failed to decode payload: {}", err);
                        return;
                    }
                };

            println!("{}", msg);
            return;
        }
        println!("Group member added successfully");
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
