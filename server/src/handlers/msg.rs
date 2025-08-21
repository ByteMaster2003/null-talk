use crate::data;
use common::{net::Packet, types::Message, utils::net::write_packet};

/// Handle a group message
/// Send the message to every active member of the group
pub async fn handle_group_message(packet: Packet, group_id: &str) {
    // Find the group chat
    let group = match data::GROUPS.lock().await.get(group_id) {
        Some(group) => group.clone(),
        None => return,
    };

    // Decode the message
    let message: Message =
        match bincode::decode_from_slice(&packet.payload, bincode::config::standard()) {
            Ok((message, _)) => message,
            Err(_) => return,
        };

    // Broadcast the message to all clients in the group
    for (member_id, is_active) in group.members {
        if !is_active {
            continue;
        }
        if member_id == message.sender_id.clone() {
            continue;
        }
        let member = match data::CLIENTS.lock().await.get(&member_id) {
            Some(member) => member.clone(),
            None => continue,
        };

        let _ = write_packet::<Packet>(member.writer.clone(), packet.clone()).await;
    }
}

// Handle a direct message
pub async fn handle_direct_message(packet: Packet, session_id: &str) {
    // Find the conversation
    let dm = match data::CONVERSATIONS.lock().await.get(session_id) {
        Some(dm) => dm.clone(),
        None => return,
    };

    // Decode the message
    let message: Message =
        match bincode::decode_from_slice(&packet.payload, bincode::config::standard()) {
            Ok((message, _)) => message,
            Err(_) => return,
        };

    // Find recipient
    let mut recipient = String::new();
    for (member, _) in &dm.members {
        if member == &message.sender_id {
            continue;
        } else {
            recipient = member.clone();
        }
    }

    // Check if client is online
    let recipient = match data::CLIENTS.lock().await.get(&recipient) {
        Some(client) => client.clone(),
        None => return,
    };

    // Send the packet to the recipient
    let _ = write_packet::<Packet>(recipient.writer.clone(), packet.clone()).await;
}
