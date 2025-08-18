use crate::{
    data,
    handlers::process_command,
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

/// ### Spawns a background task that continuously reads packets from the stream.
///
/// This function acquires a lock on the given [`StreamReader`] and runs
/// an asynchronous loop to receive incoming messages from the remote peer.
///
/// # Parameters
///
/// - `rd`: A shared reference to the reader half of the connection.
///
/// # Returns
///
/// A [`JoinHandle`] to the spawned task. The task runs until the stream
/// is closed or an unrecoverable error occurs.
///
/// # Notes
///
/// The returned handle can be awaited or aborted to manage the lifecycle
/// of the reader task.
///
/// [`StreamReader`]: common::net::StreamReader
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
                _ => (),
            }
        }
    })
}

/// ### Spawns a background task that continuously writes packets to the stream.
///
/// This function acquires a lock on the given [`StreamWriter`] and runs
/// an asynchronous loop to send outgoing messages to the remote peer.
///
/// # Parameters
///
/// - `wt`: A shared reference to the writer half of the connection.
///
/// # Returns
///
/// A [`JoinHandle`] to the spawned task. The task runs until the stream
/// is closed or an unrecoverable error occurs.
///
/// # Notes
///
/// The returned handle can be awaited or aborted to manage the lifecycle
/// of the writer task.
///
/// [`StreamWriter`]: common::net::StreamWriter
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

/// ### Spawns a background task that processes user commands and interacts with the stream.
///
/// This function drives the command-handling loop. It uses the provided
/// [`StreamReader`] and [`StreamWriter`] to perform request/response I/O,
/// while coordinating with the given reader task handle to manage state.
///
/// # Parameters
///
/// - `rd`: A shared reference to the reader half of the connection.
/// - `wt`: A shared reference to the writer half of the connection.
/// - `rd_task`: The handle to the running reader task. This can be used
///   to restart or abort the reader if necessary.
///
/// # Returns
///
/// A [`JoinHandle`] to the spawned task. The task runs until the command
/// loop is terminated or the underlying connection fails.
///
/// # Notes
///
/// This task typically serves as the "controller" of the session, binding
/// together user input, message processing, and stream I/O.
///
/// [`StreamReader`]: common::net::StreamReader
/// [`StreamWriter`]: common::net::StreamWriter
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

    let mut msg_data = Message {
        id: session.id.clone(),
        username: Some(client_config.name.clone()),
        sender_id: client_config.user_id.clone(),
        content: input.as_bytes().to_vec(),
        timestamps: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    };

    // Add the message into message list
    update_msg_list(session.id.clone(), msg_data.clone()).await;

    match encrypt_message(input, session.encryption.clone()) {
        Ok(payload) => {
            // Update encrypted message
            msg_data.content = payload;

            // Encode the payload
            let payload = match bincode::encode_to_vec(&msg_data, bincode::config::standard()) {
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

            {}
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

    let decrypted_msg = match decrypt_message(&msg.content, session.encryption.clone()) {
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
    update_msg_list(id.clone(), msg.clone()).await;
}

async fn update_msg_list(id: String, msg: Message) {
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
