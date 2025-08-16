use bincode::{Decode, Encode};

use crate::types::SymmetricAlgo;

/**
 * Represents the current chat mode (none, direct message, or group).
 */
#[derive(Clone, Debug, Encode, Decode, PartialEq)]
pub enum ChatMode {
    Dm(String),
    Group(String),
}

#[derive(Encode, Decode, PartialEq, Debug, serde::Deserialize)]
pub struct NewGroupPayload {
    pub name: String,
    pub group_id: Option<String>,
    pub members: Vec<String>,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct NewSessionPayload {
    pub id: String,
    pub mode: ChatMode,
    pub algo: SymmetricAlgo,
}

#[derive(Encode, Decode, PartialEq, Debug, serde::Deserialize)]
pub struct AddGroupMemberPayload {
    pub group_id: String,
    pub member_id: String,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct NewGroupResponse {
    pub session_key: Vec<u8>,
    pub group_id: String,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct NewSessionResponse {
    pub id: String,
    pub session_key: Vec<u8>,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct ServerResponse {
    pub success: bool,
    pub payload: Option<Vec<u8>>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
pub struct Message {
    pub id: String,               // SessionId or GroupId
    pub sender_id: String,        // UserId of sender
    pub username: Option<String>, // UserName of sender
    pub content: Vec<u8>,          // Message content
    pub timestamps: u128,          // Timestamp
}
