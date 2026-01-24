use futures::{SinkExt, StreamExt};
use lib::{
    crypto,
    protocol::{LoginPayload, OpCode, Packet, PacketCodec},
    types::ConnectionConfig,
};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

const LOGS: bool = true;
fn log(src: String) {
    if LOGS {
        println!("{src}")
    }
}

pub async fn perform_handshake(
    frames: &mut Framed<TcpStream, PacketCodec>,
    config: &ConnectionConfig,
) -> Option<Vec<u8>> {
    log(format!("[Handshake]: Sending Login Packet"));
    let _ = frames
        .send(Packet::new(
            OpCode::Login,
            LoginPayload {
                username: config.username.clone(),
                public_key: config.public_key_str.clone(),
            }
            .get_bytes(),
        ))
        .await;
    log(format!("[Handshake]: Login Packet Sent"));

    log(format!("[Handshake]: Receiving LoginAck"));
    let nonce = match frames.next().await {
        Some(Ok(pkt)) => pkt.payload,
        _ => return None,
    };
    log(format!("[Handshake]: LoginAck Received"));

    log(format!("[Handshake]: Signing nonce"));
    let signature = crypto::sign_bytes(&config.private_key, &nonce);
    log(format!("[Handshake]: Nonce Signed"));

    log(format!("[Handshake]: Sending Signature Packet"));
    let _ = frames.send(Packet::new(OpCode::Signature, signature)).await;
    log(format!("[Handshake]: Signature Packet Sent"));

    log(format!("[Handshake]: Receiving SignatureAck"));
    let session_key = match frames.next().await {
        Some(Ok(pkt)) => crypto::decrypt_bytes(&config.private_key, &pkt.payload).unwrap(),
        _ => return None,
    };
    log(format!("[Handshake]: SignatureAck Received"));

    Some(session_key)
}
