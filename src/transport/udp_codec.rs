use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

use crate::{
    common::binary::{FromBytes, ToBytes},
    encap::{Encapsulation, RawEncapsulation, header::EncapsulationHeader},
};

pub struct EncapsulationUdpCodec;

impl EncapsulationUdpCodec {
    pub fn new() -> Self {
        Self
    }
}

impl Decoder for EncapsulationUdpCodec {
    type Item = RawEncapsulation;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        log::info!("Decoding UDP datagram: {}", src.len());
        if src.len() < EncapsulationHeader::LEN {
            log::info!(
                "Not enough bytes to read header: {} < {}",
                src.len(),
                EncapsulationHeader::LEN
            );
            src.clear();
            return Ok(None);
        }

        let len_opt = EncapsulationHeader::length_from_bytes(src);
        if len_opt.is_none() {
            log::info!(
                "Not enough bytes to read length from header bytes: {}",
                src.len()
            );
            src.clear();
            return Ok(None);
        }

        let header =
            EncapsulationHeader::decode(&mut src.split_to(EncapsulationHeader::LEN).freeze())
                .map_err(|err| {
                    src.clear();
                    std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
                })?;

        Ok(Some(RawEncapsulation::new(header, src.split().freeze())))
    }
}

impl Encoder<Encapsulation> for EncapsulationUdpCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: Encapsulation, dst: &mut BytesMut) -> Result<(), Self::Error> {
        log::info!(
            "Encoding encapsulation command: {:?}, length: {}",
            item.header.command,
            item.header.length
        );

        let item_len = item.encoded_len();
        if dst.len() < item_len {
            log::warn!(
                "Not enough bytes to encode encapsulation reserving space in buffer atual: {}, required: {}",
                dst.len(),
                item_len
            );
            dst.reserve(item_len);
        }

        item.encode(dst).map_err(|err| {
            log::warn!("Failed to encode encapsulation: {}", err);
            std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
        })?;
        Ok(())
    }
}
