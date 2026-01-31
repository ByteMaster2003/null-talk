use crate::{
    data::{self, Client},
    net::{dm, perform_handshake},
    utils::types::AsyncStream,
};
use futures::{SinkExt, StreamExt};
use lib::{
    crypto,
    protocol::{OpCode, Packet, PacketCodec},
};
use tokio::sync::mpsc;
use tokio_util::codec::Framed;

pub async fn handle_client(stream: Box<dyn AsyncStream>) {
    // step 1: create lines frame fram tokio_util
    let mut frames = Framed::new(stream, PacketCodec);

    let (session_key, username, pub_key) = match perform_handshake(&mut frames).await {
        Some(d) => (d.0, d.1, d.2),
        None => return,
    };
    let public_key = crypto::parse_public_key_from_str(pub_key.clone()).unwrap();
    let user_id = crypto::public_key_to_user_id(&public_key);

    // step 3: create channel (Mail Box or Queue) for the client
    let (tx, mut rx) = mpsc::channel::<Packet>(100);

    // step 4: register the user in Dash map
    {
        data::CLIENTS.insert(
            user_id.clone(),
            Client {
                session_key: session_key,
                tx,
                user_id: user_id.clone(),
                username: username.clone(),
                public_key: pub_key,
            },
        );
    }
    println!("New Client: {}", username);

    let (mut sink, mut stream) = frames.split();

    // Task A writer task
    // read the message from channel and forward it to the client
    let writer_task = tokio::spawn(async move {
        while let Some(pkt) = rx.recv().await {
            if let Err(_) = sink.send(pkt).await {
                break; // client is probalby disconnected
            }
        }
    });

    // Task B Reader task
    // wait for the clients message and forward the message to the receivers channel
    while let Some(result) = stream.next().await {
        match result {
            Ok(pkt) => match pkt.header.op_code {
                OpCode::DmHandshake => dm::handshake(pkt, user_id.clone()).await,
                OpCode::DirectMsg => dm::direct_msg(pkt, user_id.clone()).await,
                _ => (),
            },
            _ => break,
        }
    }

    writer_task.abort();
    data::CLIENTS.remove(&user_id);
    println!("Client Disconnected: {}", username);
}
