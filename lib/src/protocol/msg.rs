use serde::{Deserialize, Serialize};
use std::io;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    pub sender_id: String, // UserId of sender
    pub username: String,  // UserName of sender
    pub content: String,   // Message content
    pub timestamps: u128,  // Timestamp
}

impl Message {
    pub fn to_bytes(self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    pub fn parse(src: &[u8]) -> Result<Self, io::Error> {
        match bincode::deserialize::<Self>(src) {
            Ok(msg) => return Ok(msg),
            Err(e) => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string()));
            }
        };
    }
}
