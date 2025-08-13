use std::path::PathBuf;

use config::{Config, File};

use crate::{
    data::SESSIONS,
    types::{ChatMode, Session},
};
use common::{
    net::{Packet, StreamReader, StreamWriter},
    types::{EncryptionConfig, NewGroupPayload, NewGroupResponse, ServerResponse, SymmetricAlgo},
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
        kind: common::net::ChatMessageKind::Command("/mkgp".to_string()),
        payload: bincode::encode_to_vec(&payload, bincode::config::standard())
            .unwrap_or("Failed to encode payload".into()),
    };

    println!("debug1");
    let mut reader = rd.lock().await;
    println!("debug2");
    if let Err(_) = netutils::write_packet(wt.clone(), packet).await {
        eprintln!("Failed to send packet");
        return None;
    }

    let response: ServerResponse = match netutils::read_packet_with_reader(&mut reader).await {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Failed to read response: {}", e);
            return None;
        }
    };
    drop(reader);

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

pub fn add_group_member(_: &str) -> Option<Session> {
    let mut s_list = SESSIONS.lock().unwrap();
    let session = s_list.get_mut("ksession")?;
    Some(session.clone())
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
