use crate::net::{HandshakePacket, StreamReader, StreamWriter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn read_handshake_packet(
    rd: StreamReader,
) -> Result<HandshakePacket, Box<dyn std::error::Error + Send + Sync>> {
    let mut reader = rd.lock().await;

    let len = reader.read_u32().await?;
    let mut buf = vec![0u8; len as usize];
    reader.read_exact(&mut buf).await?;
    let (packet, _): (HandshakePacket, usize) =
        bincode::decode_from_slice(&buf, bincode::config::standard())?;

    Ok(packet)
}

pub async fn write_handshake_packet(
    wt: StreamWriter,
    packet: HandshakePacket,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
