use std::collections::HashMap;

use common::net::StreamWriter;

/// Represents a connected client
#[derive(Clone)]
pub struct Client {
    /// username of the client
    pub username: String,
    /// user ID of the client
    pub user_id: String,
    /// session key of the client
    pub session_key: String,
    /// direct message chats the client is part of
    pub dms: Vec<String>,
    /// group chats the client is part of
    pub groups: Vec<String>,
    /// Stream writer for the client
    pub writer: StreamWriter,
}

/// Represents a direct message chat
#[derive(Debug, Clone)]
pub struct DmChat {
    /// unique identifier for the direct message chat
    pub dm_id: String,
    /// members of the direct message chat
    pub members: HashMap<String, bool>,
    /// session key for the direct message chat
    pub session_key: Vec<u8>,
}

/// Represents a group chat
#[derive(Debug, Clone)]
pub struct GroupChat {
    /// name of the group chat
    pub group_name: String,
    /// unique identifier for the group chat
    pub group_id: String,
    /// members of the group chat
    pub members: HashMap<String, bool>,
    /// session key for the group chat
    pub session_key: Vec<u8>,
    /// admin's user_id of the group chat
    pub admin: String,
}
