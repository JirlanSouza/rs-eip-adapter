use bytes::{Buf, BufMut};

use crate::{
    cip::data_types::ascii::CipAsciiExt,
    common::binary::{BinaryError, FromBytes, ToBytes},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShortString([u8; Self::BYTES_LEN]);

impl ShortString {
    pub const MAX_LEN: usize = u8::MAX as usize;
    const BYTES_LEN: usize = Self::MAX_LEN + 1;

    pub fn new(value: &str) -> Self {
        let mut storage = [0u8; Self::BYTES_LEN];
        let mut len = 0;

        for (i, c) in value.to_cip_ascii_iter().enumerate() {
            if i >= Self::MAX_LEN {
                break;
            }
            storage[i + 1] = c as u8;
            len += 1;
        }

        storage[0] = len as u8;
        Self(storage)
    }

    pub fn from_bytes(value: &[u8]) -> Self {
        let mut storage = [0u8; Self::BYTES_LEN];
        let len = usize::min(value.len(), Self::MAX_LEN);

        storage[0] = len as u8;
        storage[1..len + 1].copy_from_slice(&value[..len]);
        Self(storage)
    }

    pub fn len(&self) -> usize {
        self.0[0] as usize
    }

    pub fn value(&self) -> &str {
        if self.len() == 0 {
            return "";
        }

        unsafe { std::str::from_utf8_unchecked(&self.0[1..self.len() + 1]) }
    }
}

impl From<&str> for ShortString {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl FromBytes for ShortString {
    fn decode<T: Buf>(buffer: &mut T) -> Result<Self, BinaryError> {
        if buffer.remaining() < 1 {
            return Err(BinaryError::Truncated {
                expected: 1,
                actual: buffer.remaining(),
            });
        }

        let len = buffer.get_u8() as usize;
        if buffer.remaining() < len {
            return Err(BinaryError::Truncated {
                expected: len,
                actual: buffer.remaining(),
            });
        }

        let mut value_bytes = [0u8; Self::BYTES_LEN];
        value_bytes[0] = len as u8;
        buffer.copy_to_slice(&mut value_bytes[1..len + 1]);

        Ok(Self(value_bytes))
    }
}

impl ToBytes for ShortString {
    fn encode<T: BufMut>(&self, buffer: &mut T) -> Result<(), BinaryError> {
        if buffer.remaining_mut() < self.encoded_len() {
            return Err(BinaryError::BufferTooSmall {
                expected: self.encoded_len(),
                actual: buffer.remaining_mut(),
            });
        }
        buffer.put_slice(&self.0[0..self.encoded_len()]);
        Ok(())
    }

    fn encoded_len(&self) -> usize {
        self.0[0] as usize + 1
    }
}
