//! ### Client connection handler.
//!
//! This module is responsible for managing the lifecycle of a single
//! client connection. It provides the [`handle_client`] entry point,
//! which performs the initial handshake and spawns background tasks
//! for message I/O and command handling.
//!
//! # Overview
//!
//! - **Handshake**: Ensures the client is authentic and establishes
//!   a session key via [`perform_handshake`].
//! - **Tasks**:
//!   - Reader task: continuously reads incoming client messages.
//!   - Writer task: sends queued messages to the client.
//!   - Command task: processes commands and coordinates between
//!     reader and writer tasks.
//! - **Shutdown**: when the command task finishes, the writer task
//!   is aborted and the connection is cleaned up.
//!
//! # Related Modules
//!
//! - [`crate::handlers::task`] – Provides implementations for the
//!   reader, writer, and command tasks.
//! - [`crate::utils`] – Contains utilities like [`perform_handshake`].
//! - [`crate::types`] – Defines types such as [`LogMessage`] and [`LogLevel`].

use crate::{
    handlers::task,
    types::{LogLevel, LogMessage},
    utils::perform_handshake,
};
use common::net::AsyncStream;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Handles an incoming client connection.
///
/// This function performs the handshake with the client, sets up
/// the necessary background tasks (reader, writer, and command handler),
/// and manages the lifecycle of the connection until termination.
///
/// # Workflow
///
/// 1. Splits the provided [`TcpStream`] into a reader and writer half,
///    wrapped in [`Arc`] + [`tokio::sync::Mutex`] for shared access.
/// 2. Performs a handshake with the client using [`perform_handshake`].
///    - On failure, logs an error via [`LogMessage`] and terminates early.
/// 3. Spawns three asynchronous tasks:
///    - **Writer task**: Sends outgoing messages to the client.
///    - **Reader task**: Receives incoming messages from the client.
///    - **Command task**: Orchestrates user commands and coordinates with
///      the reader and writer tasks.
/// 4. Awaits the command task until completion, and aborts the writer
///    task when shutting down.
///
/// # Parameters
///
/// - `stream`: The accepted [`TcpStream`] for the connected client.
///
/// # Notes
///
/// - The reader and writer halves are stored in [`Arc<Mutex<...>>`] so
///   that multiple tasks can access them safely.
/// - If the handshake fails, the connection is closed immediately.
/// - When the command task ends, the writer task is aborted to clean up.
///
/// [`TcpStream`]: tokio::net::TcpStream
/// [`perform_handshake`]: crate::utils::perform_handshake
/// [`LogMessage`]: crate::types::LogMessage
pub async fn handle_client(stream: Box<dyn AsyncStream>) {
    let (rd, wt) = tokio::io::split(stream);
    let rd = Arc::new(Mutex::new(rd));
    let wt = Arc::new(Mutex::new(wt));

    let _ = match perform_handshake(rd.clone(), wt.clone()).await {
        Ok(session_key) => session_key,
        Err(e) => {
            let _ = LogMessage::log(LogLevel::ERROR, format!("Handshake failed: {}", e), 0).await;
            return;
        }
    };

    // 2. Message Writer Transmitter Task
    let wt_task = task::start_writer_task(wt.clone()).await;

    // 1. Task to receive incoming messages
    let rd_task = task::start_reader_task(rd.clone()).await;

    // 3. Command Handler Task
    let cmd_task = task::start_command_task(rd.clone(), wt.clone(), rd_task).await;

    let _ = cmd_task.await;
    wt_task.abort();
}
