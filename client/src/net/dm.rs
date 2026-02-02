use crate::{
    data,
    types::{HandshakeStatus, LogLevel, LogMessage, Message, Session},
};
use lib::{
    crypto,
    protocol::{self, DMessage, DmHandshakePayload, DmHandshakeStage, OpCode, Packet},
};
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn new_dm(user_id: String) -> Option<Session> {
    let config = data::CLIENT.get().unwrap();
    let client_session_key = data::SESSION_KEY.get().unwrap().clone();

    let id_1 = format!("{}{}", config.user_id, &user_id);
    let id_2 = format!("{}{}", &user_id, config.user_id);
    let hash_1 = crypto::hash_string(&id_1);
    let hash_2 = crypto::hash_string(&id_2);

    let sessions = data::SESSIONS.clone();
    let dm_1 = sessions.get(&hash_1);
    let dm_2 = sessions.get(&hash_2);
    let (dm_key, _) = crypto::generate_session_data();

    let dm_session = match dm_1.or(dm_2) {
        Some(session) => session.clone(),
        None => {
            let session = Session {
                status: HandshakeStatus::Requested,
                name: None,
                user_id: None,
                public_key: None,
                enc_key: dm_key,
                id: hash_1.clone(),
            };
            sessions.insert(hash_1.clone(), session.clone());

            {
                data::MESSAGES.entry(hash_1.clone()).or_insert(Vec::new());
            }

            session
        }
    };

    let payload = DmHandshakePayload {
        stage: DmHandshakeStage::Request,
        dm_id: dm_session.id.clone(),
        user_id: user_id.clone(),
        username: None,
        public_key: None,
        dm_key: vec![],
        signature: vec![],
        error: None,
        timestamps: get_timestamps(),
    };
    let payload_bytes = protocol::to_bytes::<DmHandshakePayload>(&payload);
    let enc_payload = match crypto::encrypt_aes(&client_session_key, &payload_bytes) {
        Ok(p) => p,
        Err(e) => {
            let _ = LogMessage::log(LogLevel::ERROR, format!("Error: {}", e.to_string()), 0).await;
            return None;
        }
    };
    let pkt = Packet::new(OpCode::DmHandshake, enc_payload);
    let _ = data::CHANNELS.get().unwrap().pkt_tx.send(pkt).await;

    Some(dm_session)
}

pub async fn handshake(pkt: Packet) {
    let client_session_key = match data::SESSION_KEY.get() {
        Some(key) => key.clone(),
        None => {
            let _ = LogMessage::log(LogLevel::ERROR, "Session Key Not Found!".to_string(), 0).await;
            return;
        }
    };

    let enc_bytes = pkt.payload;
    let dec_bytes = match crypto::decrypt_aes(&client_session_key, &enc_bytes) {
        Ok(bytes) => bytes,
        Err(e) => {
            let _ = LogMessage::log(LogLevel::ERROR, e.to_string(), 5).await;
            return;
        }
    };
    let handshake_data = match protocol::parse::<DmHandshakePayload>(&dec_bytes) {
        Ok(data) => data,
        Err(e) => {
            let _ = LogMessage::log(LogLevel::ERROR, e.to_string(), 5).await;
            return;
        }
    };

    match handshake_data.stage {
        DmHandshakeStage::RequestAck => request_ack(handshake_data).await,
        DmHandshakeStage::SessionAck => session_ack(handshake_data).await,
        DmHandshakeStage::SuccessAck => success_ack(handshake_data).await,
        DmHandshakeStage::Error => {
            let err_message = handshake_data
                .error
                .unwrap_or("Something went wrong! Please try again".to_string());
            let _ = LogMessage::log(LogLevel::ERROR, err_message, 0).await;
        }
        _ => (),
    }
}

async fn request_ack(data: DmHandshakePayload) {
    let config = data::CLIENT.get().unwrap().clone();
    let client_session_key = data::SESSION_KEY.get().unwrap().clone();

    let pub_key = match data.public_key {
        Some(key) => key,
        None => {
            let _ = LogMessage::log(LogLevel::ERROR, "Public key not found".to_string(), 0).await;
            return;
        }
    };
    let public_key = match crypto::parse_public_key_from_str(pub_key.clone()) {
        Ok(data) => data,
        Err(e) => {
            let _ = LogMessage::log(LogLevel::ERROR, e.to_string(), 0).await;
            return;
        }
    };
    let user_id = crypto::public_key_to_user_id(&public_key);
    let dm_key = data.dm_id.clone();

    let mut dm_session = match data::SESSIONS.get(&dm_key) {
        Some(session) => session.clone(),
        None => {
            let _ = LogMessage::log(LogLevel::ERROR, String::from("Session Not Found!"), 0).await;
            return;
        }
    };
    dm_session.name = data.username;
    dm_session.public_key = Some(pub_key.clone());
    dm_session.user_id = Some(user_id.clone());
    dm_session.status = HandshakeStatus::PubKeyReceived;

    data::SESSIONS.insert(dm_session.id.clone(), dm_session.clone());

    // Send the encrypted dm_key;
    let dm_key = crypto::encrypt_bytes(&public_key, &dm_session.enc_key);
    let signature = crypto::sign_bytes(&config.private_key, &dm_key);
    let payload = DmHandshakePayload {
        stage: DmHandshakeStage::Session,
        dm_id: dm_session.id,
        user_id,
        dm_key,
        signature,

        public_key: None,
        username: None,
        error: None,
        timestamps: get_timestamps(),
    };

    let bytes = protocol::to_bytes::<DmHandshakePayload>(&payload);
    let enc_payload = match crypto::encrypt_aes(&client_session_key, &bytes) {
        Ok(p) => p,
        Err(e) => {
            let _ = LogMessage::log(LogLevel::ERROR, e.to_string(), 0).await;
            return;
        }
    };
    let pkt = Packet::new(OpCode::DmHandshake, enc_payload);
    let _ = data::CHANNELS.get().unwrap().pkt_tx.send(pkt).await;
}

async fn session_ack(data: DmHandshakePayload) {
    let config = data::CLIENT.get().unwrap();
    let client_session_key = data::SESSION_KEY.get().unwrap().clone();

    // Parse the public key
    let pub_key = match data.public_key.clone() {
        Some(key) => key,
        _ => return,
    };
    let public_key = match crypto::parse_public_key_from_str(pub_key.clone()) {
        Ok(data) => data,
        _ => return,
    };
    let user_id = crypto::public_key_to_user_id(&public_key);

    // Verify the signature
    let sig = crypto::verify_signature(&public_key, &data.dm_key, &data.signature);
    if !sig {
        return;
    }

    // Decrypt the session key
    let dec_dm_key = match crypto::decrypt_bytes(&config.private_key, &data.dm_key) {
        Ok(key) => key,
        _ => return,
    };

    // Add the new session to the list
    let dm_id = data.dm_id.clone();
    {
        data::SESSIONS.insert(
            dm_id.clone(),
            Session {
                status: HandshakeStatus::Success,
                name: data.username.clone(),
                user_id: Some(user_id.clone()),
                public_key: data.public_key,
                enc_key: dec_dm_key,
                id: dm_id.clone(),
            },
        );
    }
    {
        data::MESSAGES.entry(dm_id.clone()).or_insert(Vec::new());
    }

    // Send the HandShake success packet
    let payload = DmHandshakePayload {
        stage: DmHandshakeStage::Success,
        dm_id: dm_id.clone(),
        user_id: user_id.clone(),

        username: data.username,
        public_key: None,
        signature: vec![],
        dm_key: vec![],
        error: None,
        timestamps: get_timestamps(),
    };
    let bytes = protocol::to_bytes::<DmHandshakePayload>(&payload);
    let enc_payload = match crypto::encrypt_aes(&client_session_key, &bytes) {
        Ok(p) => p,
        Err(e) => {
            let _ = LogMessage::log(LogLevel::INFO, e.to_string(), 0).await;
            return;
        }
    };
    let pkt = Packet::new(OpCode::DmHandshake, enc_payload);
    let _ = data::CHANNELS.get().unwrap().pkt_tx.send(pkt).await;
}

async fn success_ack(data: DmHandshakePayload) {
    let dm_id = data.dm_id;

    let mut dm_session = match data::SESSIONS.get(&dm_id) {
        Some(s) => s.clone(),
        None => {
            let _ = LogMessage::log(LogLevel::ERROR, String::from("Session Not Found!"), 0).await;
            return;
        }
    };
    dm_session.status = HandshakeStatus::Success;
    data::SESSIONS.insert(dm_id.clone(), dm_session);
}

pub async fn direct_msg(pkt: Packet) {
    let client_session_key = {
        let key = data::SESSION_KEY.get().unwrap();
        key.clone()
    };

    let enc_bytes = pkt.payload;
    let dec_bytes = match crypto::decrypt_aes(&client_session_key, &enc_bytes) {
        Ok(bytes) => bytes,
        Err(e) => {
            let _ = LogMessage::log(LogLevel::ERROR, format!("DMessage1: {}", e), 5).await;
            return;
        }
    };
    let message = match protocol::parse::<DMessage>(&dec_bytes) {
        Ok(data) => data,
        Err(e) => {
            let _ = LogMessage::log(LogLevel::ERROR, format!("DMessage2: {}", e), 5).await;
            return;
        }
    };

    if let Some(error) = message.error {
        let _ = LogMessage::log(LogLevel::ERROR, format!("DMessage: {}", error), 5).await;
        return;
    };

    let dm_id = message.id;
    let dm_session = match data::SESSIONS.get(&dm_id) {
        Some(s) => s.clone(),
        None => return,
    };

    let dec_msg_bytes = match crypto::decrypt_aes(&dm_session.enc_key, &message.content) {
        Ok(msg) => msg,
        _ => return,
    };
    let msg_content = match protocol::parse::<String>(&dec_msg_bytes) {
        Ok(m) => m,
        _ => return,
    };

    if let Some(mut mgs) = data::MESSAGES.get_mut(&dm_id) {
        mgs.push(Message {
            id: dm_id.clone(),
            user_id: dm_session.user_id.unwrap(),
            username: dm_session.name.unwrap(),
            content: msg_content,
            timestamps: message.timestamps,
        });
    }
}

pub async fn send_dm(msg: String, session: String) {
    let config = data::CLIENT.get().unwrap();
    let client_session_key = data::SESSION_KEY.get().unwrap().clone();
    let dm_session = match data::SESSIONS.get(&session) {
        Some(s) => s.clone(),
        None => {
            let _ = LogMessage::log(LogLevel::ERROR, String::from("Session not found!"), 0).await;
            return;
        }
    };

    let timestamps = get_timestamps();
    if let Some(mut mgs) = data::MESSAGES.get_mut(&session) {
        mgs.push(Message {
            id: session.clone(),
            user_id: config.user_id.clone(),
            username: config.username.clone(),
            content: msg.clone(),
            timestamps: timestamps.clone(),
        });
    }

    // Encrypt the message content with dm-key
    let msg_content = protocol::to_bytes::<String>(&msg);
    let enc_bytes = match crypto::encrypt_aes(&dm_session.enc_key, &msg_content) {
        Ok(enc) => enc,
        Err(e) => {
            let _ = LogMessage::log(LogLevel::ERROR, e.to_string(), 0).await;
            return;
        }
    };

    // prepare the message object
    let message = DMessage {
        id: session.clone(),
        user_id: dm_session.user_id.clone().unwrap(),
        content: enc_bytes.clone(),
        timestamps,
        error: None,
    };

    // Encrypt the payload with client session key
    let msg_bytes = protocol::to_bytes(&message);
    let enc_payload = match crypto::encrypt_aes(&client_session_key, &msg_bytes) {
        Ok(enc) => enc,
        Err(e) => {
            let _ = LogMessage::log(LogLevel::ERROR, e.to_string(), 0).await;
            return;
        }
    };

    // Send the packet
    let pkt = Packet::new(OpCode::DirectMsg, enc_payload);
    let _ = data::CHANNELS.get().unwrap().pkt_tx.send(pkt).await;
}

fn get_timestamps() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}
