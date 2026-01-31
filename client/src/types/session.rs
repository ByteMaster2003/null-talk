#[derive(Clone, Debug, PartialEq)]
pub enum HandshakeStatus {
    Requested,
    PubKeyReceived,
    Success,
}

#[derive(Clone, Debug)]
pub struct Session {
    pub status: HandshakeStatus,
    pub name: Option<String>,
    pub user_id: Option<String>,
    pub public_key: Option<String>,
    pub enc_key: Vec<u8>,
    pub id: String,
}

#[derive(Clone, Debug)]
pub struct Message {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub content: String,
    pub timestamps: u128,
}
