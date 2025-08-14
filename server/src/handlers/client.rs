use tokio::{io::AsyncWriteExt, sync::mpsc};

use crate::{
    data::CLIENTS,
    handlers::{handle_direct_message, handle_group_message},
    process_command,
    types::Client,
    utils::perform_handshake,
};
use common::{
    net::{ChatMessageKind, Packet, StreamReader, StreamWriter},
    types::ServerResponse,
    utils::{
        enc::public_key_to_user_id,
        net::{read_packet, write_packet},
    },
};

pub async fn handle_client(
    rd: StreamReader,
    wt: StreamWriter,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (name, session_key, public_key) = match perform_handshake(rd.clone(), wt.clone()).await {
        Ok((name, session_key, public_key)) => (name, session_key, public_key),
        Err(e) => return Err(format!("Handshake failed: {}", e).into()),
    };

    let (_tx, mut rx): (
        mpsc::UnboundedSender<String>,
        mpsc::UnboundedReceiver<String>,
    ) = mpsc::unbounded_channel();

    let client_id = public_key_to_user_id(&public_key);
    let client = Client {
        username: name.clone().to_string(),
        user_id: client_id.clone(),
        session_key: hex::encode(&session_key),
        writer: wt.clone(),
    };

    // Add new client to the server◊
    {
        let mut clients_lock = CLIENTS.lock().unwrap();
        clients_lock.insert(client_id.clone(), client);
    }
    println!("🔗 New client connected: {}", &client_id[..8]);

    // Spawn writer task: it will own the write-half access pattern
    let w_clone = wt.clone();
    let writer_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Some(msg) => {
                    // lock, write, drop immediately
                    let mut guard = w_clone.lock().await;
                    if let Err(e) = guard.write_all(msg.as_bytes()).await {
                        eprintln!("writer error: {}", e);
                        break;
                    }
                    // guard dropped here
                }
                None => break, // channel closed
            }
        }
    });

    // Spawn reader task: it will read frames and broadcast (example skeleton)
    let reader_task = start_reader_task(rd.clone(), wt.clone(), client_id.clone()).await;

    // Wait for reader task to finish and then tidy up
    // you can join/await tasks or handle as needed
    let _ = reader_task.await;
    // ensure writer task stops too
    writer_task.abort();

    println!("🔗 client disconnected: {}", &client_id[..8]);

    // Remove client from map
    {
        let mut clients_lock = CLIENTS.lock().unwrap();
        clients_lock.remove(&client_id);
    }

    Ok(())
}

async fn start_reader_task(
    rd: StreamReader,
    wt: StreamWriter,
    client_id: String,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let packet: Packet = match read_packet(rd.clone()).await {
                Ok(packet) => packet,
                Err(_) => break,
            };

            match packet.kind.clone() {
                ChatMessageKind::Command(cmd) => {
                    let response = process_command(packet.payload, client_id.clone(), &cmd).await;
                    let _ = write_packet::<ServerResponse>(wt.clone(), response).await;
                }
                ChatMessageKind::DirectMessage(receiver_id) => {
                    let _ = handle_direct_message(packet.clone(), &receiver_id, &client_id).await;
                }
                ChatMessageKind::GroupMessage(group_id) => {
                    let _ = handle_group_message(packet.clone(), &group_id, &client_id).await;
                }
            }
        }
    })
}
