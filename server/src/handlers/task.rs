use std::sync::Arc;

use common::{
    net::{ChatMessageKind, Packet, StreamReader, StreamWriter},
    types::ServerResponse,
    utils::net::{read_packet, write_packet},
};
use tokio::{
    sync::{
        Mutex as AsyncMutex,
        mpsc::{UnboundedReceiver, UnboundedSender},
    },
    task::JoinHandle,
};

use crate::{
    handlers::{handle_direct_message, handle_group_message},
    process_command,
};

pub async fn start_writer_task(mut rx: UnboundedReceiver<Packet>) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Some(packet) => match packet.kind.clone() {
                    ChatMessageKind::DirectMessage(id) => {
                        handle_direct_message(packet, &id).await;
                    }
                    ChatMessageKind::GroupMessage(id) => {
                        handle_group_message(packet, &id).await;
                    }
                    _ => {}
                },
                None => break, // channel closed
            }
        }
    })
}

pub async fn start_reader_task(
    rd: StreamReader,
    wt: StreamWriter,
    id: String,
    tx: Arc<AsyncMutex<UnboundedSender<Packet>>>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let packet: Packet = match read_packet(rd.clone()).await {
                Ok(packet) => packet,
                Err(_) => break,
            };

            match packet.kind.clone() {
                ChatMessageKind::Command(cmd) => {
                    let response = process_command(packet.payload, id.clone(), &cmd).await;
                    let _ = write_packet::<ServerResponse>(wt.clone(), response).await;
                }
                _ => {
                    let _ = tx.lock().await.send(packet);
                }
            }
        }
    })
}
