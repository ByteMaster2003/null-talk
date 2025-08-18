use std::sync::Arc;

use tokio::{
    net::TcpStream,
    sync::{Mutex as AsyncMutex, mpsc::UnboundedSender},
};

use crate::{
    data::CLIENTS, handlers::task::start_reader_task, types::Client, net::perform_handshake,
};
use common::{net::Packet, utils::enc::public_key_to_user_id};

/// Handle a new client connection
pub async fn handle_client(stream: TcpStream, tx: Arc<AsyncMutex<UnboundedSender<Packet>>>) {
    let (rd, wt) = stream.into_split();
    let rd = Arc::new(AsyncMutex::new(rd));
    let wt = Arc::new(AsyncMutex::new(wt));

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
