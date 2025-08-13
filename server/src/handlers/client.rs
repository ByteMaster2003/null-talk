use tokio::{io::AsyncWriteExt, sync::mpsc};

use crate::{
    data::CLIENTS,
    process_command,
    types::{Client, StreamReader, StreamWriter},
    utils::perform_handshake,
};
use common::{
    net::{ChatMessageKind, Packet},
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
    println!(
        "🔗 New client connected: {}",
        &client_id[..8].to_uppercase()
    );

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
    let r_clone = rd.clone();
    let w_clone = wt.clone();
    let client_id_clone = client_id.clone();
    let reader_task = tokio::spawn(async move {
        loop {
            let packet: Packet = match read_packet(r_clone.clone()).await {
                Ok(packet) => packet,
                Err(e) => {
                    eprintln!("Failed to read packet: {}", e);
                    break;
                }
            };

            match packet.kind {
                ChatMessageKind::Command(_) => {
                    match process_command(packet, client_id_clone.clone()).await {
                        Ok(res) => {
                            let _ = write_packet::<ServerResponse>(w_clone.clone(), res).await;
                        }
                        Err(e) => {
                            eprintln!("Failed to process command: {}", e);
                        }
                    }
                }
                ChatMessageKind::DirectMessage(receiver_id) => {
                    println!(
                        "Received DM from: {} to {}\nmsg: {}\n",
                        &client_id_clone[..8],
                        &receiver_id[..8],
                        String::from_utf8_lossy(&packet.payload)
                    );
                }
                ChatMessageKind::GroupMessage(group_id) => {
                    println!(
                        "Received GM from: {} to {}\nmsg: {}\n",
                        &client_id_clone[..8],
                        &group_id[..8],
                        String::from_utf8_lossy(&packet.payload)
                    );
                }
            }
        }
    });

    // Wait for reader task to finish and then tidy up
    // you can join/await tasks or handle as needed
    let _ = reader_task.await;
    // ensure writer task stops too
    writer_task.abort();

    println!("Client disconnected: {}", &client_id[..8].to_uppercase());

    // Remove client from map
    {
        let mut clients_lock = CLIENTS.lock().unwrap();
        clients_lock.remove(&client_id);
    }

    Ok(())
}
