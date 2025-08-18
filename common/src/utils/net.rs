//! Networking utilities for reading and writing packets.
//! Provides functions to read and write packets over a TCP connection.

use crate::net::{StreamReader, StreamWriter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Reads and decodes a packet from the provided stream.
///
/// This function acquires a lock on the underlying [`OwnedReadHalf`] (wrapped in
/// [`Arc`] + [`tokio::sync::Mutex`]), reads the next frame of data, and attempts to
/// deserialize it into a type `P` using [`bincode`].
///
/// # Type Parameters
///
/// * `P` - The type of the packet to decode. Must implement [`bincode::Decode`].
///
/// # Errors
///
/// Returns an error if:
/// - The stream cannot be read (e.g. due to I/O issues).
/// - The bytes cannot be decoded into `P` using [`bincode`].
///
/// # Examples
///
/// ```no_run
/// use tokio::net::TcpStream;
/// use tokio::sync::Mutex;
/// use std::sync::Arc;
/// use my_crate::read_packet;
///
/// #[derive(bincode::Decode)]
/// struct MyPacket {
///     id: u32,
///     payload: String,
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     let stream = TcpStream::connect("127.0.0.1:8080").await?;
///     let (rd, _) = stream.into_split();
///     let reader = Arc::new(Mutex::new(rd));
///
///     let packet: MyPacket = read_packet(reader).await?;
///     println!("Got packet: id={}, payload={}", packet.id, packet.payload);
///
///     Ok(())
/// }
/// ```
///
/// [`OwnedReadHalf`]: tokio::net::tcp::OwnedReadHalf
/// [`Arc`]: std::sync::Arc
/// [`tokio::sync::Mutex`]: tokio::sync::Mutex
pub async fn read_packet<P>(rd: StreamReader) -> Result<P, Box<dyn std::error::Error + Send + Sync>>
where
    P: bincode::Decode<()>,
{
    let mut reader = rd.lock().await;

    let len = reader.read_u32().await?;
    let mut buf = vec![0u8; len as usize];
    reader.read_exact(&mut buf).await?;
    let (packet, _): (P, usize) = bincode::decode_from_slice(&buf, bincode::config::standard())?;

    Ok(packet)
}

/// Encodes and writes a packet to the provided stream.
///
/// This function acquires a lock on the underlying [`OwnedWriteHalf`] (wrapped in
/// [`Arc`] + [`tokio::sync::Mutex`]), serializes the given `packet` using [`bincode`],
/// and writes the encoded bytes to the stream.
///
/// # Type Parameters
///
/// * `P` - The type of the packet to encode. Must implement [`bincode::Encode`].
///
/// # Errors
///
/// Returns an error if:
/// - The stream cannot be written to (e.g. due to I/O issues).
/// - The packet cannot be serialized by [`bincode`].
///
/// # Examples
///
/// ```no_run
/// use tokio::net::TcpStream;
/// use tokio::sync::Mutex;
/// use std::sync::Arc;
/// use my_crate::write_packet;
///
/// #[derive(bincode::Encode)]
/// struct MyPacket {
///     id: u32,
///     payload: String,
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     let stream = TcpStream::connect("127.0.0.1:8080").await?;
///     let (_, wt) = stream.into_split();
///     let writer = Arc::new(Mutex::new(wt));
///
///     let packet = MyPacket { id: 42, payload: "hello".into() };
///     write_packet(writer, packet).await?;
///
///     Ok(())
/// }
/// ```
///
/// [`OwnedWriteHalf`]: tokio::net::tcp::OwnedWriteHalf
/// [`Arc`]: std::sync::Arc
/// [`tokio::sync::Mutex`]: tokio::sync::Mutex
pub async fn write_packet<P>(
    wt: StreamWriter,
    packet: P,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    P: bincode::Encode,
{
    let mut writer = wt.lock().await;

    let encoded = bincode::encode_to_vec(packet, bincode::config::standard())?;
    writer.write_u32(encoded.len() as u32).await?;
    writer.write_all(&encoded).await?;
    writer.flush().await?;

    Ok(())
}

pub async fn close_connection(
    writer: StreamWriter,
    reason: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut w = writer.lock().await;
    let _ = w.write_all(reason.as_bytes()).await;
    let _ = w.flush().await;
    let _ = w.shutdown().await;
    Ok(())
}
