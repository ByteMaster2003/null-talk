use common::net::StreamWriter;

#[derive(Debug, Clone)]
pub struct Client {
    pub username: String,
    pub user_id: String,
    pub session_key: String,
    pub writer: StreamWriter,
}

#[derive(Debug, Clone)]
pub struct DmChat {
    pub dm_id: String,
    pub members: (String, String),
    pub session_key: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct GroupChat {
    pub group_name: String,
    pub group_id: String,

    pub members: Vec<String>,

    pub session_key: Vec<u8>,
    pub admin: String,
}
