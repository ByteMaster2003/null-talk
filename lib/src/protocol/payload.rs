use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessagePayload {
    pub sender_id: String, // UserId of sender
    pub username: String,  // UserName of sender
    pub content: String,   // Message content
    pub timestamps: u128,  // Timestamp
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct LoginPayload {
    pub username: String,
    pub public_key: String,
}
