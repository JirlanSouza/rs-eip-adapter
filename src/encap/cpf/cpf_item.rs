use bytes::{Buf, BufMut};
use std::fmt::{self, Display};

use crate::common::binary::{BinaryError, FromBytes, ToBytes};
use crate::encap::cpf::identity_item::IdentityItem;

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpfItemId {
    NullAddress = 0x0000,
    ConnectedAddress = 0x00A1,
    SequencedAddress = 0x0080,
    UnconnectedData = 0x00B2,
    ConnectedData = 0x00B1,
    IdentityItem = 0x000C,
    SockAddrInfoOtoT = 0x8000,
    SockAddrInfoTtoO = 0x8001,
}

impl TryFrom<u16> for CpfItemId {
    type Error = String;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x0000 => Ok(CpfItemId::NullAddress),
            0x00A1 => Ok(CpfItemId::ConnectedAddress),
            0x0080 => Ok(CpfItemId::SequencedAddress),
            0x00B2 => Ok(CpfItemId::UnconnectedData),
            0x00B1 => Ok(CpfItemId::ConnectedData),
            0x000C => Ok(CpfItemId::IdentityItem),
            0x8000 => Ok(CpfItemId::SockAddrInfoOtoT),
            0x8001 => Ok(CpfItemId::SockAddrInfoTtoO),
            _ => Err(format!("Invalid CpfItemId: {}", value)),
        }
    }
}

impl From<CpfItemId> for u16 {
    fn from(id: CpfItemId) -> Self {
        id as u16
    }
}

impl Display for CpfItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code: u16 = (*self).into();
        match self {
            CpfItemId::NullAddress => write!(f, "{:#06x}: NullAddress", code),
            CpfItemId::ConnectedAddress => write!(f, "{:#06x}: ConnectedAddress", code),
            CpfItemId::SequencedAddress => write!(f, "{:#06x}: SequencedAddress", code),
            CpfItemId::UnconnectedData => write!(f, "{:#06x}: UnconnectedData", code),
            CpfItemId::ConnectedData => write!(f, "{:#06x}: ConnectedData", code),
            CpfItemId::IdentityItem => write!(f, "{:#06x}: IdentityItem", code),
            CpfItemId::SockAddrInfoOtoT => write!(f, "{:#06x}: SockAddrInfoOtoT", code),
            CpfItemId::SockAddrInfoTtoO => write!(f, "{:#06x}: SockAddrInfoTtoO", code),
        }
    }
}

pub trait CpfItemDataFromBytes: Sized {
    fn decode<T: Buf>(buffer: &mut T, item_len: u16) -> Result<Self, BinaryError>;
}

pub trait CpfItemDataToBytes: ToBytes {}

#[derive(Debug, PartialEq)]
pub enum CpfItem {
    NullAddress,
    ConnectedAddress,
    SequencedAddress,
    UnconnectedData,
    ConnectedData,
    IdentityItem(Box<IdentityItem>),
    SockAddrInfoOtoT,
    SockAddrInfoTtoO,
}

impl CpfItem {
    const HEADER_LEN: usize = 4;

    pub fn id(&self) -> CpfItemId {
        match self {
            CpfItem::NullAddress => CpfItemId::NullAddress,
            CpfItem::ConnectedAddress => CpfItemId::ConnectedAddress,
            CpfItem::SequencedAddress => CpfItemId::SequencedAddress,
            CpfItem::UnconnectedData => CpfItemId::UnconnectedData,
            CpfItem::ConnectedData => CpfItemId::ConnectedData,
            CpfItem::IdentityItem(_) => CpfItemId::IdentityItem,
            CpfItem::SockAddrInfoOtoT => CpfItemId::SockAddrInfoOtoT,
            CpfItem::SockAddrInfoTtoO => CpfItemId::SockAddrInfoTtoO,
        }
    }
}

impl FromBytes for CpfItem {
    fn decode<T: Buf>(buffer: &mut T) -> Result<Self, BinaryError> {
        if buffer.remaining() < Self::HEADER_LEN {
            return Err(BinaryError::BufferTooSmall {
                expected: Self::HEADER_LEN,
                actual: buffer.remaining(),
            });
        }

        let item_id = buffer.get_u16_le();
        let item_len = buffer.get_u16_le();

        let item = match item_id {
            0x0000 => {
                buffer.advance(item_len as usize);
                CpfItem::NullAddress
            }
            0x00A1 => {
                buffer.advance(item_len as usize);
                CpfItem::ConnectedAddress
            }
            0x0080 => {
                buffer.advance(item_len as usize);
                CpfItem::SequencedAddress
            }
            0x00B2 => {
                buffer.advance(item_len as usize);
                CpfItem::UnconnectedData
            }
            0x00B1 => {
                buffer.advance(item_len as usize);
                CpfItem::ConnectedData
            }
            0x000C => CpfItem::IdentityItem(Box::new(IdentityItem::decode(buffer, item_len)?)),
            0x8000 => {
                buffer.advance(item_len as usize);
                CpfItem::SockAddrInfoOtoT
            }
            0x8001 => {
                buffer.advance(item_len as usize);
                CpfItem::SockAddrInfoTtoO
            }
            _ => {
                return Err(BinaryError::InvalidData {
                    message: "Invalid CpfItemId".to_string(),
                    expected: "Valid CpfItemId".to_string(),
                    actual: format!("0x{:04X}", item_id),
                });
            }
        };
        Ok(item)
    }
}

impl ToBytes for CpfItem {
    fn encode<T: BufMut>(&self, buffer: &mut T) -> Result<(), BinaryError> {
        buffer.put_u16_le(self.id().into());

        match self {
            CpfItem::IdentityItem(item) => {
                buffer.put_u16_le(item.encoded_len() as u16);
                item.encode(buffer)
            }
            _ => {
                buffer.put_u16_le(0);
                Ok(())
            }
        }
    }

    fn encoded_len(&self) -> usize {
        match self {
            CpfItem::IdentityItem(item) => Self::HEADER_LEN + item.encoded_len(),
            _ => Self::HEADER_LEN,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cip::data_types::short_string::ShortString;
    use bytes::BytesMut;

    #[test]
    fn cpf_item_id_mapping() {
        assert_eq!(u16::from(CpfItemId::NullAddress), 0x0000);
        assert_eq!(u16::from(CpfItemId::IdentityItem), 0x000C);

        assert_eq!(CpfItemId::try_from(0x0000).unwrap(), CpfItemId::NullAddress);
        assert_eq!(
            CpfItemId::try_from(0x000C).unwrap(),
            CpfItemId::IdentityItem
        );
        assert!(CpfItemId::try_from(0xFFFF).is_err());
    }

    #[test]
    fn cpf_item_null_address_symmetry() {
        let item = CpfItem::NullAddress;
        let mut buffer = BytesMut::new();
        item.encode(&mut buffer).expect("Failed to encode");

        assert_eq!(buffer.as_ref(), &[0x00, 0x00, 0x00, 0x00]); // ID 0, Len 0

        let mut cursor = buffer.freeze();
        let decoded = CpfItem::decode(&mut cursor).expect("Failed to decode");
        assert_eq!(decoded, item);
    }

    #[test]
    fn cpf_item_unconnected_data_symmetry() {
        let item = CpfItem::UnconnectedData;
        let mut buffer = BytesMut::new();
        item.encode(&mut buffer).expect("Failed to encode");

        assert_eq!(buffer.as_ref(), &[0xB2, 0x00, 0x00, 0x00]); // ID 0xB2, Len 0

        let mut cursor = buffer.freeze();
        let decoded = CpfItem::decode(&mut cursor).expect("Failed to decode");
        assert_eq!(decoded, item);
    }

    #[test]
    fn cpf_item_identity_item_symmetry() {
        let identity = IdentityItem {
            protocol_version: 1,
            sin_family: 2,
            sin_port: 3,
            sin_addr: 4,
            sin_zero: [0; 8],
            vendor_id: 5,
            device_type: 6,
            product_code: 7,
            revision_major: 8,
            revision_minor: 9,
            status: 10,
            serial_number: 11,
            product_name: ShortString::from("Test"),
            state: 12,
        };
        let item = CpfItem::IdentityItem(Box::new(identity));
        let mut buffer = BytesMut::new();
        item.encode(&mut buffer).expect("Failed to encode");

        let mut cursor = buffer.freeze();
        let decoded = CpfItem::decode(&mut cursor).expect("Failed to decode");
        assert_eq!(decoded, item);
    }

    #[test]
    fn cpf_item_decode_invalid_id_fails() {
        let mut buffer = BytesMut::new();
        buffer.put_u16_le(0xFFFF); // Invalid ID
        buffer.put_u16_le(0);
        let mut cursor = buffer.freeze();
        let result = CpfItem::decode(&mut cursor);
        assert!(result.is_err());
    }

    #[test]
    fn cpf_item_decode_buffer_too_small_fails() {
        let mut buffer = BytesMut::new();
        buffer.put_u16_le(0x0000); // Only part of header
        let mut cursor = buffer.freeze();
        let result = CpfItem::decode(&mut cursor);
        assert!(result.is_err());
    }
}
