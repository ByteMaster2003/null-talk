use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessagePayload {
    pub sender_id: String, // UserId of sender
    pub username: String,  // UserName of sender
    pub content: String,   // Message content
    pub timestamps: u128,  // Timestamp
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct HandshakePayload {
    pub username: String,
    pub public_key: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum DmHandshakeStage {
    Request,
    RequestAck,

    Session,
    SessionAck,

    Success,
    SuccessAck,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DmHandshakePayload {
    pub stage: DmHandshakeStage,
    pub dm_id: String,
    pub user_id: String,

    pub username: Option<String>,
    pub public_key: Option<String>,

    pub dm_key: Vec<u8>,
    pub signature: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DMessage {
    pub id: String,
    pub user_id: String,
    pub content: Vec<u8>,
    pub timestamps: u128,
}
