use std::sync::Arc;

use bincode::{Decode, Encode};
use tokio::{
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    sync::Mutex,
};

#[derive(Encode, Decode, PartialEq, Debug)]
pub enum ChatMessageKind {
    Command(String),
    DirectMessage(String),
    GroupMessage(String),
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct Packet {
    pub kind: ChatMessageKind,
    pub data: Vec<u8>,
}

pub struct ServerResponse {
    pub success: bool,
    pub message: String,
    pub error: String,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct HandshakePacket {
    pub step: u8,
    pub username: Option<String>,
    pub public_key: Option<String>,
    pub nonce: Option<Vec<u8>>,
    pub signature: Option<Vec<u8>>,
    pub session_key: Option<Vec<u8>>,
}

pub type StreamReader = Arc<Mutex<OwnedReadHalf>>;
pub type StreamWriter = Arc<Mutex<OwnedWriteHalf>>;
