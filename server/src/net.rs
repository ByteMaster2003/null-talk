use common::{
    net::{HandshakePacket, StreamReader, StreamWriter},
    utils::{
        enc::{generate_session_data, parse_public_key, verify_nonce_signature},
        net::{close_connection, read_packet, write_packet},
    },
};
use rsa::RsaPublicKey;

/// Perform the handshake process with the client
pub async fn perform_handshake(
    rd: StreamReader,
    wt: StreamWriter,
) -> Result<(String, Vec<u8>, RsaPublicKey), Box<dyn std::error::Error + Send + Sync>> {
    // Step: 0
    // Receive handshake packet from client with username and public_key
    let packet: HandshakePacket = read_packet(rd.clone()).await?;
    if packet.step != 0 {
        let _ = close_connection(wt.clone(), "Invalid handshake step").await;
        return Err("Invalid handshake step".into());
    }
    let user_name = match packet.username {
        Some(name) => name,
        None => return Err("❗️Missing username".into()),
    };
    let public_key = match packet.public_key {
        Some(key) => parse_public_key(&key).map_err(|_| "❗️Failed to parse public key")?,
        None => return Err("❗️Missing Public Key".into()),
    };

    // Step: 1
    // Generate session data (nonce & session_key)
    // Send handshake packet from server with nonce
    let (session_key, nonce) = generate_session_data();

    let packet = HandshakePacket {
        step: 1,
        username: None,
        public_key: None,
        nonce: Some(nonce.clone()),
        signature: None,
        session_key: None,
    };
    write_packet(wt.clone(), packet).await?;

    // Step: 2
    // Receive signature and verify it
    let packet: HandshakePacket = read_packet(rd.clone()).await?;
    if packet.step != 2 {
        let _ = close_connection(wt.clone(), "Invalid handshake step").await;
        return Err("Invalid handshake step".into());
    }
    let signature = match packet.signature {
        Some(signature) => signature,
        None => return Err("❗️Missing signature".into()),
    };

    if !verify_nonce_signature(&public_key, &nonce, &signature) {
        let _ = close_connection(wt.clone(), "Invalid signature!").await;
        return Err("Invalid signature!".into());
    }

    // Step: 3
    // Send Session Key After Successful Verification
    let packet = HandshakePacket {
        step: 3,
        username: None,
        public_key: None,
        nonce: None,
        signature: None,
        session_key: Some(session_key.clone()),
    };
    write_packet(wt.clone(), packet).await?;

    Ok((user_name, session_key, public_key))
}
