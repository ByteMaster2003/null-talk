use crate::protocol::OpCode;
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct PacketHeader {
    pub magic_byte: u8, // protocol indentifier
    pub version: u8,
    pub op_code: OpCode,
    pub payload_len: u32,
    pub id: Option<String>,
}

impl PacketHeader {
    pub fn to_bytes(self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    pub fn parse(src: &[u8]) -> Result<Self, io::Error> {
        match bincode::deserialize::<Self>(src) {
            Ok(header) => return Ok(header),
            Err(e) => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string()));
            }
        };
    }
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
                id: None,
            },
            payload,
        }
    }

    pub fn new_msg(op_code: OpCode, payload: Vec<u8>, id: String) -> Self {
        let payload_len = payload.len() as u32;

        Self {
            header: PacketHeader {
                magic_byte: 0x44, // protocol identifier
                version: 1,       // Current protocol version
                op_code,
                payload_len,
                id: Some(id),
            },
            payload,
        }
    }

    /// Helper for quick error packets
    pub fn error(message: &str) -> Self {
        Self::new(OpCode::Error, message.as_bytes().to_vec())
    }
}
