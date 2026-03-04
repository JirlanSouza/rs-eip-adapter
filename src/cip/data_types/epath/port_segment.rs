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
    use bytes::BytesMut;

    // Segment Type = Port Segment. Port Number = 2, Link Address = 6
    const DEFAULT_PORT_DEFAULT_LINK_BYTES: [u8; 2] = [0x02, 0x06];

    #[test]
    fn decode_default_port_default_link() {
        let mut data = bytes::Bytes::from(DEFAULT_PORT_DEFAULT_LINK_BYTES.as_slice());
        let segment =
            PortSegment::decode(&mut data).expect("Failed to decode default port default link");

        assert_eq!(segment.port, PortIdentifier::Default(2));
        assert_eq!(segment.link_address, LinkAddress::Default(6));
    }

    #[test]
    fn encode_default_port_default_link() {
        let segment = PortSegment {
            port: PortIdentifier::Default(2),
            link_address: LinkAddress::Default(6),
        };

        let mut buffer = BytesMut::with_capacity(segment.encoded_len());
        segment.encode(&mut buffer).unwrap();

        assert_eq!(buffer.as_ref(), &DEFAULT_PORT_DEFAULT_LINK_BYTES);
    }

    // Segment Type = Port Segment. Port Identifier is 15 indicating the Port Number is
    // specified in the next 16 bit field [12][00] (18 decimal). Link Address = 1.
    const EXTENDED_PORT_DEFAULT_LINK_BYTES: [u8; 4] = [0x0F, 0x12, 0x00, 0x01];

    #[test]
    fn decode_extended_port_default_link() {
        let mut data = bytes::Bytes::from(EXTENDED_PORT_DEFAULT_LINK_BYTES.as_slice());
        let segment =
            PortSegment::decode(&mut data).expect("Failed to decode extended port default link");

        assert_eq!(segment.port, PortIdentifier::Extended(18));
        assert_eq!(segment.link_address, LinkAddress::Default(1));
    }

    #[test]
    fn encode_extended_port_default_link() {
        let segment = PortSegment {
            port: PortIdentifier::Extended(18),
            link_address: LinkAddress::Default(1),
        };

        let mut buffer = BytesMut::with_capacity(segment.encoded_len());
        segment
            .encode(&mut buffer)
            .expect("Failed to encode extended port default link");

        assert_eq!(buffer.as_ref(), &EXTENDED_PORT_DEFAULT_LINK_BYTES);
    }

    // Segment Type = Port Segment. Multi-Byte address for TCP Port 5, Link Address
    // 130.151.137.105 (IP Address). The address is defined as a character array,
    // length of 15 bytes. The last byte in the segment is a pad byte.
    const DEFAULT_PORT_EXTENDED_LINK_ADDRESS_BYTES: [u8; 18] = [
        0x15, 0x0F, 0x31, 0x33, 0x30, 0x2E, 0x31, 0x35, 0x31, 0x2E, 0x31, 0x33, 0x37, 0x2E, 0x31,
        0x30, 0x35, 0x00,
    ];

    #[test]
    fn decode_default_port_extended_link_address() {
        let mut data = bytes::Bytes::from(DEFAULT_PORT_EXTENDED_LINK_ADDRESS_BYTES.as_slice());
        let segment =
            PortSegment::decode(&mut data).expect("Failed to decode default port extended link");

        assert_eq!(segment.port, PortIdentifier::Default(5));

        let expected_short_string = ShortString::new("130.151.137.105");
        assert_eq!(
            segment.link_address,
            LinkAddress::Extended(expected_short_string)
        );
    }

    #[test]
    fn encode_default_port_extended_link_address() {
        let segment = PortSegment {
            port: PortIdentifier::Default(5),
            link_address: LinkAddress::Extended(ShortString::new("130.151.137.105")),
        };

        let mut buffer = BytesMut::with_capacity(segment.encoded_len());
        segment
            .encode(&mut buffer)
            .expect("Failed to encode default port extended link address");

        assert_eq!(buffer.as_ref(), &DEFAULT_PORT_EXTENDED_LINK_ADDRESS_BYTES);
    }

    // Segment Type = Port Segment. Port Identifier is 15 indicating the Port Number is
    // specified in the next 16 bit field [12][00] (18 decimal) aftter the Link Address
    // field that value is 15. Link Address = 130.151.137.105 (IP Address). The address
    // is defined as a character array, length of 15 bytes. The last byte in the segment
    // is a pad byte.
    const EXTENDED_PORT_EXTENDED_LINK_ADDRESS_BYTES: [u8; 20] = [
        0x1F, 0x0F, 0x12, 0x00, 0x31, 0x33, 0x30, 0x2E, 0x31, 0x35, 0x31, 0x2E, 0x31, 0x33, 0x37,
        0x2E, 0x31, 0x30, 0x35, 0x00,
    ];

    #[test]
    fn decode_extended_port_extended_link_address() {
        let mut data = bytes::Bytes::from(EXTENDED_PORT_EXTENDED_LINK_ADDRESS_BYTES.as_slice());
        let segment =
            PortSegment::decode(&mut data).expect("Failed to decode extended port extended link");

        assert_eq!(segment.port, PortIdentifier::Extended(18));

        let expected_short_string = ShortString::new("130.151.137.105");
        assert_eq!(
            segment.link_address,
            LinkAddress::Extended(expected_short_string)
        );
    }

    #[test]
    fn encode_extended_port_extended_link_address() {
        let segment = PortSegment {
            port: PortIdentifier::Extended(18),
            link_address: LinkAddress::Extended(ShortString::new("130.151.137.105")),
        };

        let mut buffer = BytesMut::with_capacity(segment.encoded_len());
        segment
            .encode(&mut buffer)
            .expect("Failed to encode extended port extended link address");

        assert_eq!(buffer.as_ref(), &EXTENDED_PORT_EXTENDED_LINK_ADDRESS_BYTES);
    }
}
