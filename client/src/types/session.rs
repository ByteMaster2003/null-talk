use common::types::{ChatMode, EncryptionConfig, Message};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex as AsyncMutex;

/// Represents a chat session in the application.
///
/// A [`Session`] holds metadata and configuration about an active
/// chat context, including encryption details, operating mode,
/// and a unique identifier.
#[derive(Clone, Debug)]
pub struct Session {
    /// The display name of the session (e.g., user or group name).
    pub name: String,
    /// The encryption configuration used for securing this session.
    pub encryption: EncryptionConfig,
    /// The chat mode (e.g., direct message or group chat).
    pub mode: ChatMode,
    /// A unique identifier for the session.
    pub id: String,
}

/// ### A shared reference to the currently active [`Session`].
///
/// Wrapped in [`Arc`] and [`tokio::sync::Mutex`], this allows
/// multiple tasks to coordinate access to the current session.
/// The inner value is an [`Option`], which is `None` if no
/// session is active.
///
/// Useful when the application needs to switch between sessions
/// but only one can be active at a time.
pub type ActiveSession = Arc<AsyncMutex<Option<Session>>>;

/// ### A shared collection of all known chat sessions.
///
/// The key is the session's unique ID, and the value is the
/// corresponding [`Session`]. Wrapped in [`Arc`] and
/// [`tokio::sync::Mutex`] for safe concurrent access.
pub type Sessions = Arc<AsyncMutex<HashMap<String, Session>>>;

/// ### A shared buffer of chat messages for the application.
///
/// Stores a vector of [`Message`] values, wrapped in [`Arc`]
/// and [`tokio::sync::Mutex`] to allow concurrent producers
/// and consumers to push and read messages safely.
pub type Messages = Arc<AsyncMutex<Vec<Message>>>;
