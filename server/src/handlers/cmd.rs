use std::collections::HashMap;

use crate::{
    data::{CLIENTS, CONVERSATIONS, GROUPS},
    types::{DmChat, GroupChat},
};
use common::{
    types::{
        AddGroupMemberPayload, ChatMode, NewGroupPayload, NewGroupResponse, NewSessionPayload,
        NewSessionResponse, ServerResponse,
    },
    utils::enc::{generate_session_data, hash_string},
};
use uuid::Uuid;

/// Process a command from a client
pub async fn process_command(payload: Vec<u8>, client_id: String, cmd: &str) -> ServerResponse {
    match cmd {
        "mkgp" => create_new_group(payload, client_id.clone()).await,
        "addgpm" => add_group_member(payload, client_id.clone()).await,
        "new" => create_new_session(payload, client_id).await,
        _ => ServerResponse {
            success: false,
            payload: None,
            error: Some("Unknown Command".to_string()),
        },
    }
}

/// Create a new session
/// It can be a group or direct message
async fn create_new_session(payload: Vec<u8>, client_id: String) -> ServerResponse {
    let mut response = ServerResponse {
        success: true,
        payload: None,
        error: None,
    };

    let (new_session, _): (NewSessionPayload, usize) =
        match bincode::decode_from_slice(&payload, bincode::config::standard()) {
            Ok(data) => data,
            Err(err) => {
                response.success = false;
                response.error = Some(format!("Failed to decode payload: {}", err));

                return response;
            }
        };

    let (session_id, session_key) = match new_session.mode {
        ChatMode::Dm(_) => {
            match CLIENTS.lock().await.get(&new_session.id) {
                Some(client) => client,
                None => {
                    response.success = false;
                    response.error = Some("Member not online".to_string());

                    return response;
                }
            };

            let mut session_id: String =
                hash_string(&format!("{}{}", client_id.clone(), new_session.id.clone()));
            let (mut session_key, _) = generate_session_data();

            {
                let session_id2 =
                    hash_string(&format!("{}{}", new_session.id.clone(), client_id.clone()));

                let mut conversations = CONVERSATIONS.lock().await;
                let dm1 = conversations.get(&session_id).cloned();
                let dm2 = conversations.get(&session_id2).cloned();

                match dm1.or(dm2) {
                    Some(dm) => {
                        session_id = dm.dm_id.clone();
                        session_key = dm.session_key.clone();
                    }
                    None => {
                        let mut members: HashMap<String, bool> = HashMap::new();
                        members.insert(client_id.clone(), true);
                        members.insert(new_session.id.clone(), true);

                        for member in members.keys() {
                            let mut guard = CLIENTS.lock().await;
                            if let Some(client) = guard.get_mut(member) {
                                if !client.dms.contains(&session_id) {
                                    client.dms.push(session_id.clone());
                                }
                            }
                        }

                        let dm_chat = DmChat {
                            dm_id: session_id.clone(),
                            session_key: session_key.clone(),
                            members,
                        };

                        conversations
                            .entry(session_id.clone())
                            .or_insert(dm_chat.clone());
                    }
                }
            }

            (session_id, session_key)
        }
        ChatMode::Group(_) => {
            let group = match GROUPS.lock().await.get(&new_session.id) {
                Some(group) => group.to_owned(),
                None => {
                    response.success = false;
                    response.error = Some("Group not found".to_string());

                    return response;
                }
            };

            if !group.members.contains_key(&client_id) {
                response.success = false;
                response.error = Some("You are not a member of this group".to_string());

                return response;
            }

            let session_id = group.group_id.clone();
            let session_key = group.session_key.clone();

            {
                let mut guard = CLIENTS.lock().await;
                if let Some(client) = guard.get_mut(&client_id) {
                    if !client.groups.contains(&session_id) {
                        client.groups.push(session_id.clone());
                    }
                }
            }
            {
                let mut groups = GROUPS.lock().await;
                if let Some(group) = groups.get_mut(&session_id) {
                    group.members.insert(client_id.clone(), true);
                }
            }

            (session_id, session_key)
        }
    };

    let response_payload = NewSessionResponse {
        id: session_id,
        session_key,
    };

    response.payload =
        Some(bincode::encode_to_vec(response_payload, bincode::config::standard()).unwrap());

    return response;
}

/// Create a new group
/// If the group already exists, it won't create another group
async fn create_new_group(payload: Vec<u8>, client_id: String) -> ServerResponse {
    let (mut session_key, _) = generate_session_data();

    let mut response = ServerResponse {
        success: true,
        payload: None,
        error: None,
    };

    let mut members: HashMap<String, bool> = HashMap::new();

    let (group_info, _): (NewGroupPayload, usize) =
        match bincode::decode_from_slice(&payload, bincode::config::standard()) {
            Ok(data) => data,
            Err(err) => {
                response.success = false;
                response.error = Some(format!("Failed to decode payload: {}", err));

                return response;
            }
        };

    for member in &group_info.members {
        members.insert(member.clone(), false);
    }
    members.insert(client_id.clone(), true);

    let group_id = group_info
        .group_id
        .unwrap_or_else(|| hash_string(&Uuid::new_v4().to_string()));
    let mut new_group = GroupChat {
        group_name: group_info.name,
        group_id: group_id.clone(),
        session_key: session_key.clone(),
        admin: client_id.clone(),
        members: members.clone(),
    };

    {
        let mut groups = GROUPS.lock().await;

        match groups.get(&group_id) {
            Some(group) => {
                if group.admin != client_id {
                    response.success = false;
                    response.error = Some(format!("Group with ID {} already exists", group_id));

                    return response;
                } else {
                    new_group.session_key = group.session_key.clone();
                    session_key = group.session_key.clone();
                }
            }
            None => {
                groups.insert(group_id.clone(), new_group.clone());
            }
        }
    }

    for member in members.keys() {
        let mut guard = CLIENTS.lock().await;
        if let Some(client) = guard.get_mut(member) {
            if !client.groups.contains(&group_id) {
                client.groups.push(group_id.clone());
            }
        }
    }

    let res_payload = NewGroupResponse {
        group_id,
        session_key,
    };

    response.payload =
        Some(bincode::encode_to_vec(&res_payload, bincode::config::standard()).unwrap());

    return response;
}

/// Add a member to an existing group
async fn add_group_member(payload: Vec<u8>, client_id: String) -> ServerResponse {
    let mut response = ServerResponse {
        success: true,
        payload: None,
        error: None,
    };

    let (data, _): (AddGroupMemberPayload, usize) =
        match bincode::decode_from_slice(&payload, bincode::config::standard()) {
            Ok(data) => data,
            Err(err) => {
                response.success = false;
                response.error = Some(format!("Failed to decode payload: {}", err));

                return response;
            }
        };

    let mut group = match GROUPS.lock().await.get(&data.group_id) {
        Some(group) => group.to_owned(),
        None => {
            response.success = false;
            response.error = Some("Group not found".to_string());
            return response;
        }
    };

    if group.admin != client_id {
        response.success = false;
        response.error = Some("Only group admin can add members".to_string());
        return response;
    }

    match CLIENTS.lock().await.get(&data.member_id).cloned() {
        Some(mut member) => {
            if !group.members.contains_key(&member.user_id) {
                group.members.insert(member.user_id.clone(), true);
            }
            member.groups.push(group.group_id.clone());
        }
        None => {
            if !group.members.contains_key(&data.member_id) {
                group.members.insert(data.member_id.clone(), false);
            }
        }
    };

    response.payload = Some(
        bincode::encode_to_vec("Member Added successfully", bincode::config::standard()).unwrap(),
    );

    return response;
}
