use lib::{
    crypto,
    protocol::{self, DMessage, DmHandshakePayload, DmHandshakeStage, OpCode, Packet, parse},
};

use crate::data::{self, Client};

pub async fn handshake(pkt: Packet, user_id: String) {
    let client = {
        match data::CLIENTS.get(&user_id) {
            Some(c) => c.value().clone(),
            _ => return,
        }
    };

    let enc_payload = pkt.payload;
    let dec_payload = match crypto::decrypt_aes(&client.session_key, &enc_payload) {
        Ok(dec) => dec,
        _ => return,
    };
    let data = match parse::<DmHandshakePayload>(&dec_payload) {
        Ok(id) => id,
        _ => return,
    };

    let recv_client = {
        match data::CLIENTS.get(&data.user_id.clone()) {
            Some(c) => c.value().clone(),
            _ => return,
        }
    };

    match data.stage {
        DmHandshakeStage::Request => request(data, client, recv_client).await,
        DmHandshakeStage::Session => session(data, client, recv_client).await,
        DmHandshakeStage::Success => success(data, client, recv_client).await,
        _ => (),
    }
}

async fn request(data: DmHandshakePayload, client: Client, target_client: Client) {
    let payload = DmHandshakePayload {
        stage: DmHandshakeStage::RequestAck,
        dm_id: data.dm_id.clone(),
        user_id: target_client.user_id,

        username: Some(target_client.username),
        public_key: Some(target_client.public_key),
        signature: vec![],
        dm_key: vec![],
    };
    let bytes = protocol::to_bytes::<DmHandshakePayload>(&payload);
    let enc_payload = match crypto::encrypt_aes(&client.session_key, &bytes) {
        Ok(p) => p,
        _ => return,
    };
    let pkt = Packet::new(OpCode::DmHandshake, enc_payload);
    let _ = client.tx.send(pkt).await;
}

async fn session(data: DmHandshakePayload, client: Client, recv_client: Client) {
    let payload = DmHandshakePayload {
        stage: DmHandshakeStage::SessionAck,
        dm_id: data.dm_id.clone(),
        user_id: client.user_id.clone(),
        dm_key: data.dm_key,
        signature: data.signature,

        public_key: Some(client.public_key),
        username: Some(client.username),
    };
    let bytes = protocol::to_bytes::<DmHandshakePayload>(&payload);
    let enc_payload = match crypto::encrypt_aes(&recv_client.session_key, &bytes) {
        Ok(p) => p,
        _ => return,
    };
    let pkt = Packet::new(OpCode::DmHandshake, enc_payload);
    let _ = recv_client.tx.send(pkt).await;
}

async fn success(data: DmHandshakePayload, client: Client, recv_client: Client) {
    let payload = DmHandshakePayload {
        stage: DmHandshakeStage::SuccessAck,
        dm_id: data.dm_id.clone(),
        user_id: client.user_id.clone(),

        username: Some(client.username),
        public_key: None,
        signature: vec![],
        dm_key: vec![],
    };
    let bytes = protocol::to_bytes::<DmHandshakePayload>(&payload);
    let enc_payload = match crypto::encrypt_aes(&recv_client.session_key, &bytes) {
        Ok(p) => p,
        _ => return,
    };
    let pkt = Packet::new(OpCode::DmHandshake, enc_payload);
    let _ = recv_client.tx.send(pkt).await;
}

pub async fn direct_msg(pkt: Packet, user_id: String) {
    let client = {
        match data::CLIENTS.get(&user_id) {
            Some(c) => c.value().clone(),
            _ => return,
        }
    };

    let enc_payload = pkt.payload.clone();
    let dec_payload = match crypto::decrypt_aes(&client.session_key, &enc_payload) {
        Ok(dec) => dec,
        _ => return,
    };
    let data = match parse::<DMessage>(&dec_payload) {
        Ok(msg) => msg,
        _ => return,
    };

    let recv_client = {
        match data::CLIENTS.get(&data.user_id.clone()) {
            Some(c) => c.value().clone(),
            _ => return,
        }
    };

    let bytes = protocol::to_bytes::<DMessage>(&data);
    let enc_payload = match crypto::encrypt_aes(&recv_client.session_key, &bytes) {
        Ok(p) => p,
        _ => return,
    };
    let pkt = Packet {
        header: pkt.header,
        payload: enc_payload,
    };
    let _ = recv_client.tx.send(pkt).await;
}
