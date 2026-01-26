use crate::protocol::OpCode;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct PacketHeader {
    pub magic_byte: u8, // protocol indentifier
    pub version: u8,
    pub op_code: OpCode,
    pub payload_len: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Packet {
    pub header: PacketHeader,
    pub payload: Vec<u8>,
}

impl Packet {
    /// Creates a new packet with standard header defaults
    pub fn new(op_code: OpCode, payload: Vec<u8>) -> Self {
        let payload_len = payload.len() as u32;

        Self {
            header: PacketHeader {
                magic_byte: 0x44, // protocol identifier
                version: 1,       // Current protocol version
                op_code,
                payload_len,
            },
            payload,
        }
    }

    /// Helper for quick error packets
    pub fn error(message: &str) -> Self {
        Self::new(OpCode::Error, message.as_bytes().to_vec())
    }
}
