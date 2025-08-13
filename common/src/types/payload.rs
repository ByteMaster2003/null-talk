use bincode::{Decode, Encode};

#[derive(Encode, Decode, PartialEq, Debug, serde::Deserialize)]
pub struct NewGroupPayload {
    pub name: String,
    pub members: Vec<String>,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct ServerResponse {
    pub success: bool,
    pub payload: Option<Vec<u8>>,
    pub error: Option<String>,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct NewGroupResponse {
    pub session_key: Vec<u8>,
    pub group_id: String,
}
