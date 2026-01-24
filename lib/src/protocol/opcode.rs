use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum OpCode {
    // --- Session / Auth ---
    Login = 0x01,    // Payload: {PubKey, Signature, UserID}
    LoginAck = 0x02, // Payload: {Status}
    Signature = 0x03,
    SignatureAck = 0x04,

    // --- Messaging ---
    DirectMsg = 0x10, // Payload: {TargetUserID, IV, EncryptedBytes}
    GroupMsg = 0x11,  // Payload: {GroupID, IV, EncryptedBytes}

    // --- Group Management ---
    CreateGroup = 0x20,    // Payload: {List<UserID>}
    CreateGroupAck = 0x21, // Payload: {GroupID}
    JoinGroup = 0x22,      // Payload: {GroupID}

    // --- Errors ---
    Error = 0xFF,
}
