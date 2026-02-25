use bytes::{Buf, BufMut};

use crate::common::binary::{BinaryError, FromBytes, ToBytes};
use crate::encap::cpf::cpf_item::CpfItem;

pub mod cpf_item;
pub mod identity_item;

#[derive(Debug, PartialEq)]
pub struct Cpf {
    pub items: Vec<CpfItem>,
}

impl Cpf {
    const HEADER_LEN: usize = 2;

    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add_item(&mut self, item: CpfItem) {
        self.items.push(item);
    }
}

impl FromBytes for Cpf {
    fn decode<T: Buf>(buffer: &mut T) -> Result<Self, BinaryError> {
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
