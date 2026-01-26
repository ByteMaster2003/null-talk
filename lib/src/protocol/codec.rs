use crate::protocol::{Packet, PacketHeader, parse, to_bytes};
use std::io;
use tokio_util::{
    bytes::{Buf, BytesMut},
    codec::{Decoder, Encoder},
};

// Protocol: [ [header_len], [header_bytes], [payload_bytes]]
pub struct PacketCodec;
const HEADER_BYTE: usize = 1;

impl Decoder for PacketCodec {
    type Item = Packet;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < HEADER_BYTE {
            return Ok(None);
        }

        let header_len = src[0] as usize;
        let header_size = HEADER_BYTE + header_len;
        if src.len() < header_size {
            return Ok(None);
        }
        let header_bytes = &src[1..header_size];
        let header: PacketHeader = parse(&header_bytes)?;

        if header.magic_byte != 0x44 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid Packet"));
        }

        let payload_len = header.payload_len as usize;
        let packet_size = HEADER_BYTE + header_len + payload_len;
        if src.len() < packet_size {
            src.reserve(packet_size - src.len());
            return Ok(None);
        }

        src.advance(header_size);
        let payload_bytes = src.split_to(payload_len);

        Ok(Some(Packet {
            header,
            payload: payload_bytes.to_vec(),
        }))
    }
}

impl Encoder<Packet> for PacketCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Packet, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let header_bytes = to_bytes(&item.header);
        let payload_bytes = item.payload;

        dst.extend([header_bytes.len() as u8]); // 1 byte
        dst.extend_from_slice(&header_bytes);
        dst.extend_from_slice(&payload_bytes);

        Ok(())
    }
}
