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
