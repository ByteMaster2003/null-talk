use crate::{
    data::{CLIENTS, CONVERSATIONS, GROUPS},
    handlers::task::start_reader_task,
    net::perform_handshake,
    types::Client,
};
use common::{
    net::{AsyncStream, Packet, StreamReader, StreamWriter},
    utils::enc::public_key_to_user_id,
};
use std::sync::Arc;
use tokio::sync::{Mutex as AsyncMutex, mpsc::UnboundedSender};

/// Handle a new client connection
pub async fn handle_client(
    stream: Box<dyn AsyncStream>,
    tx: Arc<AsyncMutex<UnboundedSender<Packet>>>,
) {
    let (rd, wt) = tokio::io::split(stream);
    let rd: StreamReader = Arc::new(AsyncMutex::new(rd));
    let wt: StreamWriter = Arc::new(AsyncMutex::new(wt));

    let (name, session_key, public_key) = match perform_handshake(rd.clone(), wt.clone()).await {
        Ok(data) => data,
        Err(_) => return,
    };

    let client_id = public_key_to_user_id(&public_key);
    let client = Client {
        username: name.clone().to_string(),
        user_id: client_id.clone(),
        session_key: hex::encode(&session_key),
        dms: Vec::new(),
        groups: Vec::new(),
        writer: wt.clone(),
    };

    // Add new client to the server
    {
        let mut clients_lock = CLIENTS.lock().await;
        clients_lock.insert(client_id.clone(), client);
    }
    println!("🔗 New client connected: {}", &client_id[..8]);

    // Spawn reader task
    let read_task = start_reader_task(rd.clone(), wt.clone(), client_id.clone(), tx.clone()).await;
    let _ = read_task.await;

    println!("🔗 client disconnected: {}", &client_id[..8]);
    drop(rd);
    drop(wt);
    cleanup_client_data(client_id.clone()).await;
}

/// Removes user from groups and update/remove DM Session, etc.
async fn cleanup_client_data(client_id: String) {
    // Get client from CLIENTS
    let client = {
        let clients_lock = CLIENTS.lock().await;
        clients_lock.get(&client_id).cloned()
    };

    match client {
        Some(client) => {
            for dm in client.dms {
                update_or_remove_dm_session(&dm, client_id.clone()).await;
            }
            for gp in client.groups {
                update_or_remove_group_session(&gp, client_id.clone()).await;
            }
        }
        None => {
            println!("⚠️ Client not found for cleanup: {}", &client_id[..8]);
        }
    }
    // Additional cleanup logic can be added here

    // Remove client from CLIENTS
    {
        let mut clients_lock = CLIENTS.lock().await;
        clients_lock.remove(&client_id);
    };
}

async fn update_or_remove_dm_session(session_id: &str, client_id: String) {
    let mut conv = CONVERSATIONS.lock().await;
    let need_to_remove = match conv.get_mut(session_id) {
        Some(dm) => {
            dm.members.insert(client_id.clone(), false);
            // Check if we need to remove the DM session
            dm.members.values().all(|is_active| !is_active)
        }
        None => {
            println!("⚠️ DM session not found for ID: {}", session_id);
            false
        }
    };

    if need_to_remove {
        conv.remove(session_id);
    }
}

async fn update_or_remove_group_session(session_id: &str, client_id: String) {
    let mut groups = GROUPS.lock().await;
    let need_to_remove = match groups.get_mut(session_id) {
        Some(group) => {
            group.members.insert(client_id.clone(), false);

            // Check if we need to remove the group session
            group.members.values().all(|is_active| !is_active)
        }
        None => {
            println!("⚠️ Group session not found for ID: {}", session_id);
            false
        }
    };

    if need_to_remove {
        groups.remove(session_id);
    }
}
