use std::sync::Arc;

use bincode::{Decode, Encode};
use tokio::{
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    sync::Mutex,
};

/// Represents the kind of chat message
/// This is used to differentiate between different types of messages
/// String is going to be a command or unique identifier
#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub enum ChatMessageKind {
    /// Represents a command message
    Command(String),
    /// Represents a direct message
    DirectMessage(String),
    /// Represents a group message
    GroupMessage(String),
}

/// Represents a chat network packet
#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub struct Packet {
    /// The kind of message
    pub kind: ChatMessageKind,
    /// The payload of the message
    pub payload: Vec<u8>,
}

/// Represents a handshake packet
/// This is used to initiate a connection between clients
#[derive(Encode, Decode, PartialEq, Debug)]
pub struct HandshakePacket {
    /// The current step of the handshake
    pub step: u8,
    /// The username of the client
    pub username: Option<String>,
    /// The public key of the client
    pub public_key: Option<String>,
    /// The nonce used in the handshake
    pub nonce: Option<Vec<u8>>,
    /// The signature of the handshake
    pub signature: Option<Vec<u8>>,
    /// The session key for the handshake
    pub session_key: Option<Vec<u8>>,
}

/// Represents a stream reader for a client
pub type StreamReader = Arc<Mutex<OwnedReadHalf>>;
/// Represents a stream writer for a client
pub type StreamWriter = Arc<Mutex<OwnedWriteHalf>>;
