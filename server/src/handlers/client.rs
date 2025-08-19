use std::sync::Arc;

use tokio::sync::{Mutex as AsyncMutex, mpsc::UnboundedSender};

use crate::{
    data::CLIENTS, handlers::task::start_reader_task, net::perform_handshake, types::Client,
};
use common::{
    net::{AsyncStream, Packet, StreamReader, StreamWriter},
    utils::enc::public_key_to_user_id,
};

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

    // Remove client from map
    {
        let mut clients_lock = CLIENTS.lock().await;
        clients_lock.remove(&client_id);
    }
}
