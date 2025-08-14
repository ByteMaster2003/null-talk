use common::{net::Packet, utils::net::write_packet};

use crate::data::{self, CONVERSATIONS};

pub async fn handle_group_message(
    packet: Packet,
    group_id: &str,
    client_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Broadcast the message to all clients in the group
    let group = match data::GROUPS.lock().unwrap().get(group_id) {
        Some(group) => group.clone(),
        None => return Err("Group not found".into()),
    };

    for member_id in group.members {
        if member_id == client_id {
            continue;
        }
        let member = match data::CLIENTS.lock().unwrap().get(&member_id) {
            Some(member) => member.clone(),
            None => continue,
        };

        let _ = write_packet::<Packet>(member.writer.clone(), packet.clone()).await;
    }

    Ok(())
}

pub async fn handle_direct_message(
    packet: Packet,
    session_id: &str,
    client_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Find the recipient client
    let dm = match CONVERSATIONS.lock().unwrap().get(session_id) {
        Some(dm) => dm.clone(),
        None => return Err("Direct message not found".into()),
    };

    let member1 = dm.members.0;
    let member2 = dm.members.1;
    let recipient: String;

    if client_id == member1 {
        recipient = member2;
    } else {
        recipient = member1;
    }

    let recipient = match data::CLIENTS.lock().unwrap().get(&recipient) {
        Some(client) => client.clone(),
        None => return Err("User not found".into()),
    };

    // Send the packet to the recipient
    let _ = write_packet::<Packet>(recipient.writer.clone(), packet.clone()).await;

    Ok(())
}
