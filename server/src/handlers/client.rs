use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::mpsc,
};

use crate::{
    data::CLIENTS,
    process_command,
    types::{Client, StreamReader, StreamWriter},
    utils::perform_handshake,
};
use common::utils::enc::public_key_to_user_id;

pub async fn handle_client(
    rd: StreamReader,
    wt: StreamWriter,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (name, session_key, public_key) = match perform_handshake(rd.clone(), wt.clone()).await {
        Ok((name, session_key, public_key)) => (name, session_key, public_key),
        Err(e) => return Err(format!("Handshake failed: {}", e).into()),
    };

    let (tx, mut rx) = mpsc::unbounded_channel();

    let client_id = public_key_to_user_id(&public_key);
    let client = Client {
        username: name.clone().to_string(),
        user_id: client_id.clone(),
        session_key: hex::encode(&session_key),
        writer: tx,
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
        let mut buf = [0u8; 1024];
        loop {
            // lock only for the read call
            let mut guard = r_clone.lock().await;
            let n = match guard.read(&mut buf).await {
                Ok(0) => {
                    // connection closed
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    eprintln!("read error: {}", e);
                    break;
                }
            };
            drop(guard); // release read lock immediately

            let msg_prefix = String::from_utf8_lossy(&buf[..10]).to_string();
            if msg_prefix.starts_with("/") {
                process_command(buf, client_id_clone.clone(), w_clone.clone()).await;
            } else {
                // broadcast to others (example, not optimized)
                let msg = String::from_utf8_lossy(&buf[..n]).to_string();
                let clients_lock = CLIENTS.lock().unwrap();
                for (other_id, c) in clients_lock.iter() {
                    if *other_id != client_id_clone {
                        let _ = c.writer.send(msg.clone());
                    }
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
