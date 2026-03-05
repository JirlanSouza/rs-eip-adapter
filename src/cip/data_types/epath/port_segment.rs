use std::net::Ipv4Addr;

use bytes::{Buf, BufMut};

use crate::{
    cip::data_types::{Byte, epath::SegmentType, short_string::ShortString},
    common::binary::{BinaryError, FromBytes, ToBytes},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortIdentifier {
    Default(u8),
    Extended(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkAddress {
    Default(u8),
    Extended(ShortString),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortSegment {
    pub port: PortIdentifier,
    pub link_address: LinkAddress,
}

impl PortSegment {
    pub const MIN_LEN: usize = 2;

    pub fn from_port_and_ip(port: u8, ip: Ipv4Addr) -> Self {
        let link_address = LinkAddress::Extended(ShortString::new(ip.to_string().as_str()));
        Self {
            port: PortIdentifier::Default(port),
            link_address,
        }
    }
}

impl FromBytes for PortSegment {
    fn decode<T: bytes::Buf>(buffer: &mut T) -> Result<Self, BinaryError> {
        let mut total_len = Self::MIN_LEN;

        if buffer.remaining() < total_len {
            return Err(BinaryError::Truncated {
                expected: total_len,
                actual: buffer.remaining(),
            });
        }

        let first_byte = Byte::decode(buffer)?;
        let segment_type = SegmentType::from(first_byte.get_bits(5, 7));

        if segment_type != SegmentType::PortSegment {
            return Err(BinaryError::InvalidData {
                message: "Invalid segment type".to_string(),
                expected: SegmentType::PortSegment.to_string(),
                actual: segment_type.to_string(),
            });
        }

        let is_extended_link_address_size = first_byte.get_bit(4);
        let mut link_address_size = 1u8;

        if is_extended_link_address_size {
            link_address_size = buffer.get_u8();
            total_len += link_address_size as usize;
        }

        let port_identifier_raw = first_byte.get_bits(0, 3);
        let port_identifier: PortIdentifier;

        if port_identifier_raw == 15 {
            if buffer.remaining() < 2 {
                return Err(BinaryError::Truncated {
                    expected: 2,
                    actual: buffer.remaining(),
                });
            }

            port_identifier = PortIdentifier::Extended(buffer.get_u16_le());
            total_len += 2;
        } else {
            port_identifier = PortIdentifier::Default(port_identifier_raw);
        }

        if is_extended_link_address_size {
            if buffer.remaining() < link_address_size as usize {
                return Err(BinaryError::Truncated {
                    expected: link_address_size as usize,
                    actual: buffer.remaining(),
                });
            }

            let mut link_address_buffer = [0u8; 255];
            buffer.copy_to_slice(&mut link_address_buffer[..link_address_size as usize]);

            let link_address = LinkAddress::Extended(ShortString::from_bytes(
                &link_address_buffer[..link_address_size as usize],
            ));

            if total_len % 2 != 0 {
                if buffer.remaining() >= 1 {
                    let _pad = buffer.get_u8();
                } else {
                    return Err(BinaryError::Truncated {
                        expected: total_len + 1,
                        actual: total_len,
                    });
                }
            }

            return Ok(Self {
                port: port_identifier,
                link_address,
            });
        }

        if buffer.remaining() < 1 {
            return Err(BinaryError::Truncated {
                expected: 1,
                actual: buffer.remaining(),
            });
        }

        let link_address = LinkAddress::Default(buffer.get_u8());

        Ok(Self {
            port: port_identifier,
            link_address,
        })
    }
}

impl ToBytes for PortSegment {
    fn encode<T: BufMut>(&self, buffer: &mut T) -> Result<(), BinaryError> {
        if buffer.remaining_mut() < self.encoded_len() {
            return Err(BinaryError::BufferTooSmall {
                expected: self.encoded_len(),
                actual: buffer.remaining_mut(),
            });
        }

        let mut first_byte = Byte::new(0);
        first_byte.set_bits(5, 7, SegmentType::PortSegment as u8);
        let is_extended_link = matches!(self.link_address, LinkAddress::Extended(_));

        if is_extended_link {
            first_byte.set_bit(4);
        }

        let mut total_len = 2;
        match self.port {
            PortIdentifier::Default(port) => {
                first_byte.set_bits(0, 3, port);
            }
            PortIdentifier::Extended(_) => {
                first_byte.set_bits(0, 3, 0x0F);
                total_len += 2;
            }
        }

        buffer.put_u8(first_byte.value());

        if is_extended_link {
            if let LinkAddress::Extended(link) = &self.link_address {
                buffer.put_u8(link.len() as u8);
            }
        }

        if let PortIdentifier::Extended(port) = self.port {
            buffer.put_u16_le(port);
        }

        match &self.link_address {
            LinkAddress::Default(link) => {
                buffer.put_u8(*link);
            }
            LinkAddress::Extended(link) => {
                let link_bytes = link.value().as_bytes();
                buffer.put_slice(link_bytes);
                total_len += link_bytes.len();

                if total_len % 2 != 0 {
                    buffer.put_u8(0);
                }
            }
        }

        Ok(())
    }

    fn encoded_len(&self) -> usize {
        let mut len = 1;
        if matches!(self.link_address, LinkAddress::Extended(_)) {
            len += 1;
        }
        if matches!(self.port, PortIdentifier::Extended(_)) {
            len += 2;
        }
        match &self.link_address {
            LinkAddress::Default(_) => {
                len += 1;
            }
            LinkAddress::Extended(link) => {
                len += link.len();
                if len % 2 != 0 {
                    len += 1;
                }
            }
        }
        len
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{Bytes, BytesMut};

    #[test]
    fn decode_and_encode_default_port_default_link_symmetry() {
        let raw_bytes: [u8; 2] = [
            0x02, // Segment Type 0x00 | Port 2
            0x06, // Link Address 6
        ];

        let mut cursor = Bytes::copy_from_slice(&raw_bytes);
        let decoded = PortSegment::decode(&mut cursor).expect("Failed to decode");

        assert_eq!(decoded.port, PortIdentifier::Default(2));
        assert_eq!(decoded.link_address, LinkAddress::Default(6));

        // Round-trip Symmetry
        let mut buffer = BytesMut::with_capacity(decoded.encoded_len());
        decoded.encode(&mut buffer).expect("Failed to encode");
        assert_eq!(
            buffer.as_ref(),
            &raw_bytes,
            "Inconsistent encode/decode symmetry"
        );
    }

    #[test]
    fn decode_and_encode_extended_port_default_link_symmetry() {
        let raw_bytes: [u8; 4] = [
            0x0F, // Segment Type 0x00 | Port Extended (15)
            0x12, 0x00, // Port 18 (LE)
            0x01, // Link Address 1
        ];

        let mut cursor = Bytes::copy_from_slice(&raw_bytes);
        let decoded = PortSegment::decode(&mut cursor).expect("Failed to decode");

        assert_eq!(decoded.port, PortIdentifier::Extended(18));
        assert_eq!(decoded.link_address, LinkAddress::Default(1));

        // Round-trip Symmetry
        let mut buffer = BytesMut::with_capacity(decoded.encoded_len());
        decoded.encode(&mut buffer).expect("Failed to encode");
        assert_eq!(
            buffer.as_ref(),
            &raw_bytes,
            "Inconsistent encode/decode symmetry"
        );
    }

    #[test]
    fn decode_and_encode_default_port_extended_link_address_padded_symmetry() {
        let raw_bytes: [u8; 18] = [
            0x15, // Segment Type 0x00 | Ext Link (bit 4) | Port 5
            0x0F, // Link Address Size (15)
            0x31, 0x33, 0x30, 0x2E, 0x31, 0x35, 0x31, 0x2E, 0x31, 0x33, 0x37, 0x2E, 0x31,
            0x30, // 130.151.137.105 (IP Address)
            0x35, 0x00, // Pad byte
        ];

        let mut cursor = Bytes::copy_from_slice(&raw_bytes);
        let decoded = PortSegment::decode(&mut cursor).expect("Failed to decode");

        assert_eq!(decoded.port, PortIdentifier::Default(5));
        let expected_short_string = ShortString::new("130.151.137.105");
        assert_eq!(
            decoded.link_address,
            LinkAddress::Extended(expected_short_string)
        );

        // Round-trip Symmetry
        let mut buffer = BytesMut::with_capacity(decoded.encoded_len());
        decoded.encode(&mut buffer).expect("Failed to encode");
        assert_eq!(
            buffer.as_ref(),
            &raw_bytes,
            "Inconsistent encode/decode symmetry"
        );
    }

    #[test]
    fn decode_and_encode_extended_port_extended_link_address_padded_symmetry() {
        let raw_bytes: [u8; 20] = [
            0x1F, // Segment Type 0x00 | Ext Link (bit 4) | Port Extended (15)
            0x0F, // Link Address Size (15)
            0x12, 0x00, // Port 18 (LE)
            0x31, 0x33, 0x30, 0x2E, 0x31, 0x35, 0x31, 0x2E, 0x31, 0x33, 0x37, 0x2E, 0x31,
            0x30, // 130.151.137.105 (IP Address).
            0x35, 0x00, // Pad byte
        ];

        let mut cursor = Bytes::copy_from_slice(&raw_bytes);
        let decoded = PortSegment::decode(&mut cursor).expect("Failed to decode");

        assert_eq!(decoded.port, PortIdentifier::Extended(18));
        let expected_short_string = ShortString::new("130.151.137.105");
        assert_eq!(
            decoded.link_address,
            LinkAddress::Extended(expected_short_string)
        );

        // Round-trip Symmetry
        let mut buffer = BytesMut::with_capacity(decoded.encoded_len());
        decoded.encode(&mut buffer).expect("Failed to encode");
        assert_eq!(
            buffer.as_ref(),
            &raw_bytes,
            "Inconsistent encode/decode symmetry"
        );
    }

    #[test]
    fn decode_truncated_buffer_returns_error() {
        // Port segment with extended port but missing port data
        let shorter_bytes = [0x0F];
        let mut cursor = Bytes::copy_from_slice(&shorter_bytes);
        let result = PortSegment::decode(&mut cursor);
        assert!(matches!(result, Err(BinaryError::Truncated { .. })));
    }

    #[test]
    fn encode_buffer_too_small_returns_error() {
        let segment = PortSegment {
            port: PortIdentifier::Default(2),
            link_address: LinkAddress::Default(6),
        };
        let mut arr = [0u8; 1];
        let mut buffer = &mut arr[..];
        let result = segment.encode(&mut buffer);
        assert!(matches!(result, Err(BinaryError::BufferTooSmall { .. })));
    }

    #[test]
    fn decode_and_encode_port_14_no_extension_symmetry() {
        // Port 14 is the maximum value for Default port
        let raw_bytes: [u8; 2] = [0x0E, 0x01];
        let mut cursor = Bytes::copy_from_slice(&raw_bytes);
        let decoded = PortSegment::decode(&mut cursor).expect("Failed to decode port 14");
        assert_eq!(decoded.port, PortIdentifier::Default(14));

        let mut buffer = BytesMut::with_capacity(decoded.encoded_len());
        decoded.encode(&mut buffer).unwrap();
        assert_eq!(buffer.as_ref(), &raw_bytes);
    }

    #[test]
    fn decode_invalid_segment_type_returns_error() {
        // Segment type 0x01 (Logical Segment) instead of 0x00 (Port Segment)
        let raw_bytes: [u8; 2] = [0x22, 0x06]; // 001 00010
        let mut cursor = Bytes::copy_from_slice(&raw_bytes);
        let result = PortSegment::decode(&mut cursor);
        assert!(matches!(result, Err(BinaryError::InvalidData { .. })));
    }

    #[test]
    fn decode_and_encode_extended_link_address_no_padding_symmetry() {
        // Port 1 (Default), Ext Link len 2.
        // Total len: 1 (first) + 1 (size) + 2 (data) = 4 bytes (Even). No pad.
        let raw_bytes: [u8; 4] = [
            0x11, // Segment 0 | Ext Link | Port 1
            0x02, // Size 2
            0x31, 0x32, // "12"
        ];

        let mut cursor = Bytes::copy_from_slice(&raw_bytes);
        let decoded = PortSegment::decode(&mut cursor).expect("Failed to decode");

        assert_eq!(
            decoded.link_address,
            LinkAddress::Extended(ShortString::new("12"))
        );

        let mut buffer = BytesMut::with_capacity(decoded.encoded_len());
        decoded.encode(&mut buffer).unwrap();
        assert_eq!(buffer.as_ref(), &raw_bytes);
    }

    #[test]
    fn decode_and_encode_port_min_default_symmetry() {
        // Port 0 (Min Default)
        let raw_bytes: [u8; 2] = [0x00, 0x01];
        let mut cursor = Bytes::copy_from_slice(&raw_bytes);
        let decoded = PortSegment::decode(&mut cursor).expect("Failed to decode port 0");
        assert_eq!(decoded.port, PortIdentifier::Default(0));

        let mut buffer = BytesMut::with_capacity(decoded.encoded_len());
        decoded
            .encode(&mut buffer)
            .expect("Failed to encode port 0");
        assert_eq!(buffer.as_ref(), &raw_bytes);
    }

    #[test]
    fn decode_and_encode_port_max_extended_symmetry() {
        // Port 65535 (Max Extended)
        let raw_bytes: [u8; 4] = [0x0F, 0xFF, 0xFF, 0x01];
        let mut cursor = Bytes::copy_from_slice(&raw_bytes);
        let decoded = PortSegment::decode(&mut cursor).expect("Failed to decode port 65535");
        assert_eq!(decoded.port, PortIdentifier::Extended(65535));

        let mut buffer = BytesMut::with_capacity(decoded.encoded_len());
        decoded
            .encode(&mut buffer)
            .expect("Failed to encode port 65535");
        assert_eq!(buffer.as_ref(), &raw_bytes);
    }

    #[test]
    fn decode_and_encode_link_address_min_default_symmetry() {
        // Link Address 0 (Min Default)
        let raw_bytes: [u8; 2] = [0x01, 0x00];
        let mut cursor = Bytes::copy_from_slice(&raw_bytes);

        let decoded = PortSegment::decode(&mut cursor).expect("Failed to decode link 0");
        assert_eq!(decoded.link_address, LinkAddress::Default(0));

        let mut buffer = BytesMut::with_capacity(decoded.encoded_len());
        decoded
            .encode(&mut buffer)
            .expect("Failed to encode link 0");
        assert_eq!(buffer.as_ref(), &raw_bytes);
    }

    #[test]
    fn decode_and_encode_link_address_max_default_symmetry() {
        // Link Address 255 (Max Default)
        let raw_bytes: [u8; 2] = [0x01, 0xFF];
        let mut cursor = Bytes::copy_from_slice(&raw_bytes);
        let decoded = PortSegment::decode(&mut cursor).expect("Failed to decode link 255");
        
        assert_eq!(decoded.link_address, LinkAddress::Default(255));

        let mut buffer = BytesMut::with_capacity(decoded.encoded_len());
        decoded
            .encode(&mut buffer)
            .expect("Failed to encode link 255");
        assert_eq!(buffer.as_ref(), &raw_bytes);
    }
}
