use crate::{
    data,
    net::{dm, perform_handshake},
    types::{LogLevel, LogMessage},
    utils::types::AsyncStream,
};
use futures::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use lib::protocol::{OpCode, Packet, PacketCodec};
use tokio::sync::mpsc;
use tokio_util::codec::Framed;

pub async fn handle_client(stream: Box<dyn AsyncStream>, pkt_rx: mpsc::Receiver<Packet>) {
    let config = data::CLIENT.get().unwrap().clone();
    let mut frames = Framed::new(stream, PacketCodec);

    let session_key = match perform_handshake(&mut frames, &config).await {
        Some(key) => key,
        None => panic!("Something went wrong"),
    };
    let _ = data::SESSION_KEY.set(session_key.clone());

    // Split the TCP stream
    let (sink, stream) = frames.split();

    // Start writer task
    tokio::spawn(async move {
        let _ = writer_task(sink, pkt_rx).await;
    });

    // Start reader task
    let _ = tokio::spawn(async move {
        reader_task(stream).await;
    })
    .await;
}

async fn writer_task(
    mut sink: SplitSink<Framed<Box<dyn AsyncStream>, PacketCodec>, Packet>,
    mut pkt_rx: mpsc::Receiver<Packet>,
) {
    let mut shutdown_rx = {
        let channels = data::CHANNELS.get().unwrap();
        channels.shutdown_tx.subscribe()
    };

    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                break
            }
            Some(packet) = pkt_rx.recv() => {
                let _ = sink.send(packet).await;
            },
        }
    }
}

async fn reader_task(mut stream: SplitStream<Framed<Box<dyn AsyncStream>, PacketCodec>>) {
    let mut shutdown_rx = data::CHANNELS.get().unwrap().shutdown_tx.subscribe();

    loop {
        tokio::select! {
            // Biased to check shutdown first
            _ = shutdown_rx.recv() => {
                break;
            }
            result = stream.next() => {
                match result {
                    Some(Ok(pkt)) => {
                        tokio::spawn(async move {
                            match pkt.header.op_code {
                                OpCode::DmHandshake => dm::handshake(pkt).await,
                                OpCode::DirectMsg => dm::direct_msg(pkt).await,
                                _ => (),
                            }
                        });
                    }
                    Some(Err(e)) => {
                        let _ = LogMessage::log(LogLevel::ERROR,format!("Stream: {}", e), 0).await;
                    }
                    None => break, // Stream closed
                }
            }
        }
    }
}
