use bytes::{Buf, BufMut};

use crate::common::binary::{BinaryError, FromBytes, ToBytes};
use crate::encap::cpf::cpf_item::CpfItem;

pub mod cpf_item;
pub mod identity_item;

#[derive(Debug, PartialEq, Default)]
pub struct Cpf {
    pub items: Vec<CpfItem>,
}

impl Cpf {
    const HEADER_LEN: usize = 2;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_item(&mut self, item: CpfItem) {
        self.items.push(item);
    }
}

impl FromBytes for Cpf {
    fn decode<T: Buf>(buffer: &mut T) -> Result<Self, BinaryError> {
        if buffer.remaining() < Self::HEADER_LEN {
            return Err(BinaryError::BufferTooSmall {
                expected: Self::HEADER_LEN,
                actual: buffer.remaining(),
            });
        }
        let item_count = buffer.get_u16_le();
        let mut items = Vec::new();

        for _ in 0..item_count {
            let item = CpfItem::decode(buffer)?;
            items.push(item);
        }
        Ok(Self { items })
    }
}

impl ToBytes for Cpf {
    fn encode<T: BufMut>(&self, buffer: &mut T) -> Result<(), BinaryError> {
        let encoded_len = self.encoded_len();

        if buffer.remaining_mut() < encoded_len {
            return Err(BinaryError::BufferTooSmall {
                expected: encoded_len,
                actual: buffer.remaining_mut(),
            });
        }

        buffer.put_u16_le(self.items.len() as u16);
        for item in &self.items {
            item.encode(buffer)?;
        }
        Ok(())
    }

    fn encoded_len(&self) -> usize {
        self.items
            .iter()
            .map(|item| item.encoded_len())
            .sum::<usize>()
            + Self::HEADER_LEN
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cip::data_types::short_string::ShortString;
    use crate::encap::cpf::identity_item::IdentityItem;
    use bytes::BytesMut;

    #[test]
    fn cpf_new_and_add_item() {
        let mut cpf = Cpf::new();
        assert_eq!(cpf.items.len(), 0);

        cpf.add_item(CpfItem::NullAddress);
        assert_eq!(cpf.items.len(), 1);
        assert_eq!(cpf.items[0], CpfItem::NullAddress);
    }

    #[test]
    fn cpf_empty_symmetry() {
        let cpf = Cpf::new();
        let mut buffer = BytesMut::new();
        cpf.encode(&mut buffer).expect("Failed to encode");

        assert_eq!(buffer.as_ref(), &[0x00, 0x00]); // Item count 0

        let mut cursor = buffer.freeze();
        let decoded = Cpf::decode(&mut cursor).expect("Failed to decode");
        assert_eq!(decoded, cpf);
    }

    #[test]
    fn cpf_multiple_items_symmetry() {
        let mut cpf = Cpf::new();
        cpf.add_item(CpfItem::NullAddress);
        cpf.add_item(CpfItem::UnconnectedData);

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
        cpf.add_item(CpfItem::IdentityItem(Box::new(identity)));

        let mut buffer = BytesMut::new();
        cpf.encode(&mut buffer).expect("Failed to encode");

        let mut cursor = buffer.freeze();
        let decoded = Cpf::decode(&mut cursor).expect("Failed to decode");
        assert_eq!(decoded, cpf);
        assert_eq!(decoded.items.len(), 3);
    }

    #[test]
    fn cpf_decode_truncated_count_fails() {
        let mut buffer = BytesMut::new();
        buffer.put_u8(0);
        let mut cursor = buffer.freeze();
        let result = Cpf::decode(&mut cursor);
        assert!(matches!(result, Err(BinaryError::BufferTooSmall { .. })));
    }
}
