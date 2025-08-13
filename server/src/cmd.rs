use std::collections::{HashMap, hash_map::Entry};

use common::{
    net::Packet,
    types::{NewGroupPayload, NewGroupResponse, ServerResponse},
    utils::enc::{generate_session_data, hash_string},
};
use uuid::Uuid;

use crate::{
    data::{CLIENTS, GROUPS},
    types::{Client, GroupChat, GroupMember},
};

pub async fn process_command(
    packet: Packet,
    client_id: String,
) -> Result<ServerResponse, Box<dyn std::error::Error + Send + Sync>> {
    let group_id = hash_string(&Uuid::new_v4().to_string());
    let (session_key, _) = generate_session_data();

    let mut response = ServerResponse {
        success: true,
        payload: None,
        error: None,
    };

    let client: Client;
    {
        let clients = CLIENTS.lock().unwrap();
        client = match clients.get(&client_id).cloned() {
            Some(client) => client,
            None => {
                response.success = false;
                response.error = Some("Client not found".to_string());
                return Ok(response);
            }
        }
    }

    let admin = GroupMember {
        user_id: client.user_id.clone(),
        username: Some(client.username.clone()),
        writer: Some(client.writer.clone()),
    };
    let mut participants: HashMap<String, GroupMember> = HashMap::new();
    participants.insert(client.user_id.clone(), admin.clone());

    let (group_info, _): (NewGroupPayload, usize) =
        match bincode::decode_from_slice(&packet.payload, bincode::config::standard()) {
            Ok(data) => data,
            Err(err) => {
                response.success = false;
                response.error = Some(format!("Failed to decode payload: {}", err));

                return Ok(response);
            }
        };

    for member_id in group_info.members {
        let member = GroupMember {
            user_id: member_id.clone(),
            username: None,
            writer: None,
        };
        match participants.entry(member_id.clone()) {
            Entry::Occupied(entry) => Some(entry.get().clone()),
            Entry::Vacant(entry) => {
                entry.insert(member);
                participants.get(&member_id).cloned()
            }
        };
    }

    let new_group = GroupChat {
        group_name: group_info.name,
        group_id: group_id.clone(),
        session_key: session_key.clone(),
        admin,
        join_requests: HashMap::new(),
        participants,
    };

    {
        let mut groups = GROUPS.lock().unwrap();
        match groups.entry(group_id.clone()) {
            Entry::Occupied(entry) => Some(entry.get().clone()),
            Entry::Vacant(entry) => {
                entry.insert(new_group.clone());
                groups.get(&group_id).cloned()
            }
        };
    }

    let res_payload = NewGroupResponse {
        group_id,
        session_key,
    };

    response.payload =
        Some(bincode::encode_to_vec(&res_payload, bincode::config::standard()).unwrap());

    return Ok(response);
}
