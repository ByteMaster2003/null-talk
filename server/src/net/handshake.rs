use crate::utils::types::AsyncStream;
use futures::{SinkExt, StreamExt};
use lib::{
    crypto,
    protocol::{HandshakePayload, OpCode, Packet, PacketCodec, parse},
};
use tokio_util::codec::Framed;

const LOGS: bool = false;
fn log(src: String) {
    if LOGS {
        println!("{src}")
    }
}

pub async fn perform_handshake(
    frames: &mut Framed<Box<dyn AsyncStream>, PacketCodec>,
) -> Option<(Vec<u8>, String, String)> {
    log(format!("[Handshake]: Receiving Login Packet"));
    let login_pkt = match frames.next().await {
        Some(Ok(pkt)) => pkt,
        _ => return None,
    };
    log(format!("[Handshake]: Login Packet Received\n"));

    if login_pkt.header.op_code != OpCode::Login {
        return None;
    }

    let (username, pub_key) = match parse::<HandshakePayload>(&login_pkt.payload) {
        Ok(k) => (k.username, k.public_key),
        _ => return None,
    };
    let public_key = match crypto::parse_public_key_from_str(pub_key.clone()) {
        Ok(key) => key,
        Err(_) => return None,
    };
    let (session_key, nonce) = crypto::generate_session_data();

    // Send login Ack
    log(format!("[Handshake]: Sending LoginAck Packet"));
    let _ = frames
        .send(Packet::new(OpCode::LoginAck, nonce.clone()))
        .await;
    log(format!("[Handshake]: LoginAck Packet Sent\n"));

    log(format!("[Handshake]: Receiving Signature Packet"));
    let sig_pkt = match frames.next().await {
        Some(Ok(pkt)) => pkt,
        _ => return None,
    };
    log(format!("[Handshake]: Signature Packet Received"));

    if sig_pkt.header.op_code != OpCode::Signature {
        return None;
    }

    log(format!("[Handshake]: Verifying Signature"));
    let success = crypto::verify_signature(&public_key, &nonce, &sig_pkt.payload);
    if !success {
        return None;
    }
    log(format!("[Handshake]: Signature Verified\n"));

    log(format!("[Handshake]: Sending SignatureAck Packet"));
    let _ = frames
        .send(Packet::new(
            OpCode::SignatureAck,
            crypto::encrypt_bytes(&public_key, &session_key),
        ))
        .await;
    log(format!("[Handshake]: SignatureAck Packet Sent\n"));

    Some((session_key, username, pub_key))
}
