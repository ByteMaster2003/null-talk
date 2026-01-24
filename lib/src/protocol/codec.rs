use crate::protocol::{Packet, PacketHeader};
use std::io;
use tokio_util::{
    bytes::{Buf, BytesMut},
    codec::{Decoder, Encoder},
};

pub struct PacketCodec;
const HEADER_BYTE: usize = 1;

impl Decoder for PacketCodec {
    type Item = Packet;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < HEADER_BYTE {
            return Ok(None); // wait for full data to arrive
        }

        let header_len = src[0] as usize;
        let header_size = HEADER_BYTE + header_len;
        if src.len() < header_size {
            return Ok(None); // wait for full data to arrive
        }
        let header_bytes = &src[1..header_size];
        let header = PacketHeader::parse(header_bytes)?;

        // Security Check
        if header.magic_byte != 0x44 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid magic byte",
            ));
        }
        let payload_len = header.payload_len as usize;
        let packet_size = header_size + payload_len;

        if src.len() < packet_size {
            src.reserve(packet_size - src.len());
            return Ok(None); // wait for full data to arrive
        }

        src.advance(header_size);
        let payload_bytes = src.split_to(payload_len);

        let packet = Packet {
            header,
            payload: payload_bytes.to_vec(),
        };

        return Ok(Some(packet));
    }
}

impl Encoder<Packet> for PacketCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Packet, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Step 1: encode header into bytes
        let header_bytes = item.header.to_bytes();
        let payload_bytes = item.payload;

        // Step 2: Add header_len as first byte, exact 1 byte
        dst.extend([header_bytes.len() as u8]);

        // Step 3: Add header_bytes
        dst.extend_from_slice(&header_bytes);

        // Step 4: Add payload_bytes
        dst.extend_from_slice(&payload_bytes);

        Ok(())
    }
}
