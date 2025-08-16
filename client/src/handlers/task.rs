use crate::{
    cmd::process_command,
    data,
    types::{LogLevel, LogMessage},
};
use common::{
    net::{ChatMessageKind, Packet, StreamReader, StreamWriter},
    types::{ChatMode, Message},
    utils::{
        enc::{decrypt_message, encrypt_message},
        net::{read_packet, write_packet},
    },
};
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{sync::Mutex as AsyncMutex, task::JoinHandle};

pub async fn start_reader_task(rd: StreamReader) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let packet = match read_packet::<Packet>(rd.clone()).await {
                Ok(packet) => packet,
                Err(_) => break,
            };

            match packet.kind {
                ChatMessageKind::DirectMessage(id) => {
                    process_message(id.clone(), packet.payload.clone()).await;
                }
                ChatMessageKind::GroupMessage(id) => {
                    process_message(id.clone(), packet.payload.clone()).await;
                }
                _ => {
                    eprintln!("Unknown packet kind");
                }
            }
            print!("{}", get_prompt().await);
        }
    })
}

pub async fn start_writer_task(wt: StreamWriter) -> JoinHandle<()> {
    tokio::spawn(async move {
        let msg_rx = {
            let channels = data::CHANNELS.lock().await;
            channels.msg_rx.clone()
        };

        loop {
            match msg_rx.lock().await.recv().await {
                Some(msg) => send_message(wt.clone(), &msg).await,
                None => break,
            };
        }
    })
}

pub async fn start_command_task(
    rd: StreamReader,
    wt: StreamWriter,
    mut rd_task: JoinHandle<()>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let cmd_rx = {
            let channels = data::CHANNELS.lock().await;
            channels.cmd_rx.clone()
        };

        loop {
            let command = match cmd_rx.lock().await.recv().await {
                Some(cmd) => cmd,
                None => {
                    rd_task.abort();
                    break;
                }
            };

            rd_task.abort();
            process_command(&command, rd.clone(), wt.clone()).await;
            rd_task = start_reader_task(rd.clone()).await;
        }
    })
}

async fn get_prompt() -> String {
    match data::ACTIVE_SESSION.lock().await.as_ref() {
        Some(session) => format!("{:?}> ", session.mode),
        None => format!("> "),
    }
}

async fn send_message(wt: StreamWriter, input: &str) {
    let session = match data::ACTIVE_SESSION.lock().await.as_ref() {
        Some(session) => session.to_owned(),
        None => {
            LogMessage::log(LogLevel::ERROR, "No Active Session Found!".into(), 5).await;
            return;
        }
    };
    let client_config = match data::CLIENT_CONFIG.lock().await.as_ref() {
        Some(config) => config.to_owned(),
        None => {
            LogMessage::log(LogLevel::ERROR, "Failed to get client config".into(), 5).await;
            return;
        }
    };

    let kind = match session.mode.clone() {
        ChatMode::Dm(_) => ChatMessageKind::DirectMessage(session.id.clone()),
        ChatMode::Group(_) => ChatMessageKind::GroupMessage(session.id.clone()),
    };
    match encrypt_message(input, session.encryption.clone()) {
        Ok(payload) => {
            // Create Message payload
            let payload = Message {
                id: session.id.clone(),
                username: Some(client_config.name.clone()),
                sender_id: client_config.user_id.clone(),
                content: payload,
                timestamps: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis(),
            };

            // Encode the payload
            let payload = match bincode::encode_to_vec(&payload, bincode::config::standard()) {
                Ok(payload) => payload,
                Err(err) => {
                    LogMessage::log(
                        LogLevel::ERROR,
                        format!("Failed to encode message: {}", err),
                        5,
                    )
                    .await;
                    return;
                }
            };
            let packet = Packet { kind, payload };
            let _ = write_packet::<Packet>(wt.clone(), packet).await;
        }
        Err(err) => {
            LogMessage::log(
                LogLevel::ERROR,
                format!("Failed to encrypt message: {}", err),
                5,
            )
            .await;
        }
    };
}

async fn process_message(id: String, payload: Vec<u8>) {
    let (mut msg, _): (Message, usize) =
        match bincode::decode_from_slice(&payload, bincode::config::standard()) {
            Ok(decoded) => decoded,
            Err(err) => {
                LogMessage::log(
                    LogLevel::ERROR,
                    format!("Failed to decode message: {:?}", err),
                    5,
                )
                .await;
                return;
            }
        };

    let session = match data::SESSIONS.lock().await.get(&id) {
        Some(session) => session.to_owned(),
        None => return,
    };

    let decrypted_msg = match decrypt_message(&payload, session.encryption.clone()) {
        Ok(msg) => msg,
        Err(err) => {
            LogMessage::log(
                LogLevel::ERROR,
                format!("Failed to decrypt message: {}", err),
                5,
            )
            .await;
            return;
        }
    };

    msg.content = decrypted_msg.into_bytes();

    // Update message list
    {
        let mut messages = data::MESSAGES.lock().await;
        match messages.get_mut(&id) {
            Some(messages) => {
                let mut messages = messages.lock().await;
                messages.push(msg.clone());
            }
            None => {
                messages.insert(id.clone(), Arc::new(AsyncMutex::new(vec![msg.clone()])));
            }
        };
    }

    if let Some(session) = data::ACTIVE_SESSION.lock().await.as_ref() {
        if session.id.clone() == id.clone() {
            let ui_tx = {
                let channels = data::CHANNELS.lock().await;
                channels.ui_tx.clone()
            };

            let _ = ui_tx.lock().await.send(msg);
        }
    };
}
