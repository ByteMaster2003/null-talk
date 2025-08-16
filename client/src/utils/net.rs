use crate::data::CLIENT_CONFIG;
use common::{
    net::{HandshakePacket, StreamReader, StreamWriter},
    utils::{enc as encutils, net as netutils},
};
use rsa::RsaPrivateKey;

pub async fn perform_handshake(
    rd: StreamReader,
    wt: StreamWriter,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let username: String;
    let public_key: String;
    let private_key: RsaPrivateKey;

    // Get user credentials from config
    {
        let config = CLIENT_CONFIG.lock().await;
        username = match config.as_ref() {
            Some(cfg) => cfg.name.clone(),
            None => return Err("❗️Failed to get username from config".into()),
        };
        public_key = match config.as_ref() {
            Some(cfg) => encutils::to_ssh_public_key(&cfg.public_key)
                .to_openssh()
                .unwrap(),
            None => return Err("❗️Failed to get public key from config".into()),
        };
        private_key = match config.as_ref() {
            Some(cfg) => cfg.private_key.clone(),
            None => return Err("❗️Failed to get private key from config".into()),
        };
    }

    // Step: 0
    // Send handshake packet with username and public_key
    let packet = HandshakePacket {
        step: 0,
        username: Some(username),
        public_key: Some(public_key),
        nonce: None,
        signature: None,
        session_key: None,
    };
    netutils::write_packet(wt.clone(), packet).await?;

    // Step: 1
    // Receive handshake packet from server with nonce
    // Sign the nonce with private_key
    let packet: HandshakePacket = netutils::read_packet(rd.clone()).await?;
    assert!(packet.step == 1);
    let nonce = match packet.nonce {
        Some(nonce) => nonce.clone(),
        None => return Err("❗️Failed to get nonce from handshake packet".into()),
    };
    let signature = encutils::sign_nonce(&private_key, &nonce);

    // Step: 2
    // Send handshake packet with signature
    let packet = HandshakePacket {
        step: 2,
        username: None,
        public_key: None,
        nonce: None,
        signature: Some(signature),
        session_key: None,
    };
    netutils::write_packet(wt.clone(), packet).await?;

    // Step: 3
    // Receive handshake packet with session_key
    let packet: HandshakePacket = netutils::read_packet(rd.clone()).await?;
    assert!(packet.step == 3);
    let session_key = packet
        .session_key
        .ok_or("❗️Failed to get session_key from handshake packet")?;

    Ok(session_key)
}
