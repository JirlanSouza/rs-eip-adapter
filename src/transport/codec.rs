use bytes::{Buf, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::{
    common::binary::{FromBytes, ToBytes},
    encap::{Encapsulation, RawEncapsulation, header::EncapsulationHeader},
};

pub struct EncapsulationCodec;

impl EncapsulationCodec {
    pub fn new() -> Self {
        Self
    }
}

impl Decoder for EncapsulationCodec {
    type Item = RawEncapsulation;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        log::info!("Decoding TCP frame: {}", src.len());
        if src.len() < EncapsulationHeader::LEN {
            log::info!(
                "Not enough bytes to read header: {} < {}",
                src.len(),
                EncapsulationHeader::LEN
            );
            return Ok(None);
        }

        let len_opt = EncapsulationHeader::length_from_bytes(src);
        if len_opt.is_none() {
            log::info!(
                "Not enough bytes to read length from header bytes: {}",
                src.len()
            );
            return Ok(None);
        }
        let len = len_opt.unwrap();
        if src.len() < len as usize + EncapsulationHeader::LEN {
            log::info!(
                "Not enough bytes to read encapsulation: {} < {}",
                src.len(),
                len as usize + EncapsulationHeader::LEN
            );
            return Ok(None);
        }

        let header =
            EncapsulationHeader::decode(&mut src.split_to(EncapsulationHeader::LEN).freeze())
                .map_err(|err| {
                    log::info!("Failed to decode header: {}", err);
                    src.advance(EncapsulationHeader::LEN + len as usize);
                    std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
                })?;

        let encapsulation = RawEncapsulation::new(header, src.split_to(len as usize).freeze());
        Ok(Some(encapsulation))
    }
}

impl Encoder<Encapsulation> for EncapsulationCodec {
    type Error = std::io::Error;

    fn encode(&mut self, _item: Encapsulation, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        log::info!(
            "Encoding encapsulation command: {:?}, length: {}",
            _item.header.command,
            _item.header.length
        );

        let item_len = _item.encoded_len();
        if _dst.len() < item_len {
            log::warn!(
                "Not enough bytes to encode encapsulation: {} < {}",
                _dst.len(),
                item_len
            );
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Not enough bytes to encode encapsulation",
            ));
        }

        _item.encode(_dst).map_err(|err| {
            log::warn!("Failed to encode encapsulation: {}", err);
            std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
        })?;
        Ok(())
    }
}
