use std::{collections::HashMap, sync::Arc};
use tokio::{
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    sync::Mutex,
};

#[derive(Debug, Clone)]
pub struct Client {
    pub username: String,
    pub user_id: String,
    pub session_key: String,
    pub writer: StreamWriter,
}

#[derive(Debug, Clone)]
pub struct DmChat {
    pub initiator: Client,
    pub other: Client,
    pub session_key: String,
}

#[derive(Debug, Clone)]
pub struct GroupMember {
    pub username: Option<String>,
    pub user_id: String,
    pub writer: Option<StreamWriter>,
}

#[derive(Debug, Clone)]
pub struct GroupChat {
    pub group_name: String,
    pub group_id: String,

    pub participants: HashMap<String, GroupMember>,
    pub join_requests: HashMap<String, GroupMember>,

    pub session_key: Vec<u8>,
    pub admin: GroupMember,
}

pub type StreamReader = Arc<Mutex<OwnedReadHalf>>;
pub type StreamWriter = Arc<Mutex<OwnedWriteHalf>>;
