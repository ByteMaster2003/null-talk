use std::path::PathBuf;

use config::{Config, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    data::SESSIONS,
    types::{ChatMode, Session},
};
use common::{
    net::{StreamReader, StreamWriter},
    types::{EncryptionConfig, SymmetricAlgo},
    utils::file::resolve_path,
};

#[derive(serde::Deserialize)]
pub struct GroupInfo {
    pub name: String,
    pub members: Vec<String>,
}

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

    let group_info = match parse_group_file(&path) {
        Some(info) => info,
        None => {
            eprintln!("Failed to parse group file: {:?}", path);
            return None;
        }
    };

    {
        let mut writer = wt.lock().await;
        let mut packet: Vec<u8> = Vec::new();
        let command = "/mkgp".to_string();

        // Add command name
        packet.push(command.len() as u8);
        packet.extend(command.into_bytes());

        // Add group name
        packet.push(group_info.name.len() as u8);
        packet.extend(group_info.name.clone().into_bytes());

        // Add member count
        packet.push(group_info.members.len() as u8);
        for member in group_info.members {
            packet.push(member.len() as u8);
            packet.extend(member.into_bytes());
        }
        packet.push(0); // Null terminator for the member list
        writer.write_all(&packet).await.unwrap();
        writer.flush().await.unwrap();
    }

    let mut buf = vec![0u8; 2048];
    let n = {
        let mut reader = rd.lock().await;
        match reader.read(&mut buf).await {
            Ok(n) => n,
            Err(e) => {
                eprintln!("Failed to create group: {}", e);
                return None;
            }
        }
    };

    if n < 64 {
        eprintln!("Failed to create group");
        return None;
    }
    let session_key = buf[0..32].to_vec();
    let group_id_len = buf[32] as usize;
    let group_id = hex::encode(&buf[33..33 + group_id_len]);

    Some(Session {
        name: group_info.name.clone(),
        id: group_id,
        mode: ChatMode::Group(group_info.name.clone()),
        encryption: EncryptionConfig {
            algo: SymmetricAlgo::AES256,
            encryption_key: Some(session_key),
        },
    })
}

pub fn add_group_member(_: &str) -> Option<Session> {
    let mut s_list = SESSIONS.lock().unwrap();
    let session = s_list.get_mut("ksession")?;
    Some(session.clone())
}

pub fn parse_group_file(path: &PathBuf) -> Option<GroupInfo> {
    let file = File::with_name(path.to_str().unwrap());
    let cfg = Config::builder().add_source(file).build().unwrap();

    let group_info = match cfg.try_deserialize::<GroupInfo>() {
        Ok(group) => group,
        Err(_) => return None,
    };

    Some(group_info)
}
