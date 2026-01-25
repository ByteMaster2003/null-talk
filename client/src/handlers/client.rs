use futures::{SinkExt, StreamExt};
use lib::{
    crypto,
    protocol::{MessagePayload, OpCode, Packet, PacketCodec},
    types::ConnectionConfig,
};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LinesCodec};

use crate::handlers::perform_handshake;

pub async fn handle_client(stream: TcpStream, config: &ConnectionConfig) {
    let mut frames = Framed::new(stream, PacketCodec);

    let session_key = match perform_handshake(&mut frames, config).await {
        Some(key) => key,
        None => panic!("Something went wrong"),
    };

    let user_id = crypto::public_key_to_user_id(&config.public_key);

    println!("Connected Successfully");
    println!("UserId: {}", user_id.clone());
    println!();
    println!("===================================================================");
    println!("===================================================================");
    println!("\n\n");

    let (mut sink, mut stream) = frames.split();

    let session_key_clone = session_key.clone();
    let config_clone = config.clone();
    let user_id_clone = user_id.clone();

    let writer_task = tokio::spawn(async move {
        let mut io_reader = Framed::new(tokio::io::stdin(), LinesCodec::new());

        while let Some(result) = io_reader.next().await {
            match result {
                Ok(input) => {
                    let (receiver_id, msg) = input.split_once(":").unwrap();

                    let message = MessagePayload {
                        sender_id: user_id_clone.clone(),
                        content: msg.to_string(),
                        username: config_clone.username.clone(),
                        timestamps: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis(),
                    }
                    .to_bytes();

                    let encrypted_msg = crypto::encrypt_aes(&session_key_clone, &message).unwrap();
                    let _ = sink
                        .send(Packet::new_msg(
                            OpCode::DirectMsg,
                            encrypted_msg,
                            receiver_id.to_string(),
                        ))
                        .await;
                }
                Err(_) => break,
            }
        }
    });

    while let Some(result) = stream.next().await {
        match result {
            Ok(pkt) => match pkt.header.op_code {
                lib::protocol::OpCode::DirectMsg => {
                    match crypto::decrypt_aes(&session_key, &pkt.payload) {
                        Ok(msg) => {
                            if let Ok(message) = MessagePayload::parse(&msg) {
                                println!("[{}]: {}", message.username, message.content);
                            }
                        }
                        Err(e) => println!("Error: {}", e),
                    };
                }
                _ => println!("Operation Type: {:?}", &pkt.header.op_code),
            },
            Err(_) => break,
        }
    }

    writer_task.abort();
    println!("Disconnected")
}
