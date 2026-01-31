use bytes::{BufMut, BytesMut};

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

impl From<CpfItemId> for u16 {
    fn from(id: CpfItemId) -> Self {
        id as u16
    }
}

pub struct CpfEncoder<'a> {
    buffer: &'a mut BytesMut,
    count_pos: usize,
    item_count: u16,
    current_item_len_pos: Option<usize>,
}

impl<'a> CpfEncoder<'a> {
    pub fn new(buffer: &'a mut BytesMut) -> Self {
        let count_pos = buffer.len();
        buffer.put_u16_le(0);
        Self {
            buffer,
            count_pos,
            item_count: 0,
            current_item_len_pos: None,
        }
    }

    pub fn add_item_start(&mut self, item_id: CpfItemId) -> &mut BytesMut {
        self.add_item_end();
        self.buffer.put_u16_le(item_id as u16);

        self.current_item_len_pos = Some(self.buffer.len());
        self.buffer.put_u16_le(0);
        self.item_count += 1;
        self.buffer
    }

    pub fn finish(mut self) {
        self.add_item_end();
        let count_bytes = self.item_count.to_le_bytes();
        self.buffer[self.count_pos..self.count_pos + 2].copy_from_slice(&count_bytes);
    }

    fn add_item_end(&mut self) {
        if let Some(len_pos) = self.current_item_len_pos.take() {
            let data_start = len_pos + 2;
            let data_len = (self.buffer.len() - data_start) as u16;

            let len_bytes = data_len.to_le_bytes();
            self.buffer[len_pos..len_pos + 2].copy_from_slice(&len_bytes);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{BufMut, BytesMut};

    #[test]
    fn cpf_single_item_length_and_count() {
        let mut buf = BytesMut::new();
        let mut cpf_encoder = CpfEncoder::new(&mut buf);
        {
            let item = cpf_encoder.add_item_start(CpfItemId::IdentityItem);
            item.put_u8(0xAA);
            item.put_u8(0xBB);
        }
        cpf_encoder.finish();

        let items_count = u16::from_le_bytes([buf[0], buf[1]]);
        assert_eq!(items_count, 1);

        let item_id = u16::from_le_bytes([buf[2], buf[3]]);
        assert_eq!(item_id, CpfItemId::IdentityItem as u16);

        let items_len = u16::from_le_bytes([buf[4], buf[5]]);
        assert_eq!(items_len, 2);

        assert_eq!(buf[6], 0xAA);
        assert_eq!(buf[7], 0xBB);
    }

    #[test]
    fn cpf_multiple_items_and_lengths() {
        let mut buf = BytesMut::new();
        let mut cpf_encoder = CpfEncoder::new(&mut buf);
        {
            let first_item = cpf_encoder.add_item_start(CpfItemId::IdentityItem);
            first_item.put_slice(&[1, 2, 3]);
        }
        {
            let second_item = cpf_encoder.add_item_start(CpfItemId::SockAddrInfoOtoT);
            second_item.put_u8(0xFF);
        }
        cpf_encoder.finish();

        let items_count = u16::from_le_bytes([buf[0], buf[1]]);
        assert_eq!(items_count, 2);

        let first_item_id = u16::from_le_bytes([buf[2], buf[3]]);
        assert_eq!(first_item_id, CpfItemId::IdentityItem as u16);
        let first_item_len = u16::from_le_bytes([buf[4], buf[5]]);
        assert_eq!(first_item_len, 3);

        let second_item_id_offset = 6 + (first_item_len as usize);
        let second_item_id = u16::from_le_bytes([buf[second_item_id_offset], buf[second_item_id_offset + 1]]);
        assert_eq!(second_item_id, CpfItemId::SockAddrInfoOtoT as u16);
        let second_item_len = u16::from_le_bytes([buf[second_item_id_offset + 2], buf[second_item_id_offset + 3]]);
        assert_eq!(second_item_len, 1);
        let second_item_data = buf[second_item_id_offset + 4];
        assert_eq!(second_item_data, 0xFF);
    }

    #[test]
    fn cpf_zero_length_item() {
        let mut buf = BytesMut::new();
        let mut cpf_encoder = CpfEncoder::new(&mut buf);
        {
            let _ = cpf_encoder.add_item_start(CpfItemId::IdentityItem);
        }
        cpf_encoder.finish();

        let count = u16::from_le_bytes([buf[0], buf[1]]);
        assert_eq!(count, 1);
        let len = u16::from_le_bytes([buf[4], buf[5]]);
        assert_eq!(len, 0);
    }

    #[test]
    fn cpf_add_item_end_updates_previous_length() {
        let mut buf = BytesMut::new();
        let mut cpf_encoder = CpfEncoder::new(&mut buf);
        {
            let first_item = cpf_encoder.add_item_start(CpfItemId::IdentityItem);
            first_item.put_slice(&[10, 11]);
            let second_item = cpf_encoder.add_item_start(CpfItemId::ConnectedAddress);
            second_item.put_u8(0x01);
        }
        cpf_encoder.finish();

        let first_item_len = u16::from_le_bytes([buf[4], buf[5]]);
        assert_eq!(first_item_len, 2);

        let second_item_len_offset = 6 + 2 + 2;
        let second_item_len = u16::from_le_bytes([buf[second_item_len_offset], buf[second_item_len_offset + 1]]);
        assert_eq!(second_item_len, 1);
    }
}
