use crate::common::binary::{BinaryError, FromBytes, ToBytes};

pub(crate) trait CipAsciiExt {
    fn to_cip_ascii_iter(&self) -> impl Iterator<Item = char>;
}

impl CipAsciiExt for str {
    fn to_cip_ascii_iter(&self) -> impl Iterator<Item = char> {
        self.chars().map(|c| match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'A',
            'É' | 'È' | 'Ê' | 'Ë' => 'E',
            'Í' | 'Ì' | 'Î' | 'Ï' => 'I',
            'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'O',
            'Ú' | 'Ù' | 'Û' | 'Ü' => 'U',
            'Ç' => 'C',
            _ if c.is_ascii() && !c.is_ascii_control() => c,
            _ => '.',
        })
    }
}

pub trait StringLen: Sized + Copy + Into<usize> + FromBytes + ToBytes {
    fn new() -> Self;
    fn plus_one(self) -> Self;
    fn from_usize(len: usize) -> Self;
    fn into_usize(self) -> usize;
}

impl StringLen for u8 {
    fn new() -> Self {
        0
    }

    fn plus_one(self) -> Self {
        self + 1
    }

    fn from_usize(len: usize) -> Self {
        len as u8
    }

    fn into_usize(self) -> usize {
        self as usize
    }
}

impl FromBytes for u8 {
    fn decode<T: bytes::Buf>(buffer: &mut T) -> Result<Self, BinaryError> {
        if buffer.remaining() < 1 {
            return Err(BinaryError::Truncated {
                expected: 1,
                actual: buffer.remaining(),
            });
        }

        Ok(buffer.get_u8())
    }
}

impl ToBytes for u8 {
    fn encode<T: bytes::BufMut>(&self, buffer: &mut T) -> Result<(), BinaryError> {
        buffer.put_u8(*self);
        Ok(())
    }

    fn encoded_len(&self) -> usize {
        1
    }
}

impl StringLen for u16 {
    fn new() -> Self {
        0
    }

    fn plus_one(self) -> Self {
        self + 1
    }

    fn from_usize(len: usize) -> Self {
        len as u16
    }

    fn into_usize(self) -> usize {
        self as usize
    }
}

impl FromBytes for u16 {
    fn decode<T: bytes::Buf>(buffer: &mut T) -> Result<Self, BinaryError> {
        if buffer.remaining() < 2 {
            return Err(BinaryError::Truncated {
                expected: 2,
                actual: buffer.remaining(),
            });
        }

        Ok(buffer.get_u16_le())
    }
}

impl ToBytes for u16 {
    fn encode<T: bytes::BufMut>(&self, buffer: &mut T) -> Result<(), BinaryError> {
        buffer.put_u16_le(*self);
        Ok(())
    }

    fn encoded_len(&self) -> usize {
        2
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsciiString<L: StringLen, const N: usize> {
    len: L,
    characters: [u8; N],
}

impl<L: StringLen, const N: usize> AsciiString<L, N> {
    pub const MAX_LEN: usize = N;
    const BYTES_LEN: usize = Self::MAX_LEN + std::mem::size_of::<L>();

    pub fn new(value: &str) -> Self {
        let mut characters = [0u8; N];
        let mut len = L::new();

        for (i, c) in value.to_cip_ascii_iter().enumerate() {
            if i >= Self::MAX_LEN {
                break;
            }
            characters[i + 1] = c as u8;
            len = len.plus_one();
        }

        Self { len, characters }
    }

    pub fn from_bytes(value: &[u8]) -> Self {
        let mut characters = [0u8; N];
        let len = usize::min(value.len(), Self::MAX_LEN);
        characters[..len].copy_from_slice(&value[..len]);

        Self {
            len: L::from_usize(len),
            characters,
        }
    }

    pub fn len(&self) -> usize {
        self.len.into_usize()
    }

    pub fn value(&self) -> &str {
        if self.len() == 0 {
            return "";
        }

        unsafe { std::str::from_utf8_unchecked(&self.characters[..self.len()]) }
    }
}

impl<L: StringLen, const N: usize> FromBytes for AsciiString<L, N> {
    fn decode<T: bytes::Buf>(buffer: &mut T) -> Result<Self, BinaryError> {
        if buffer.remaining() < Self::BYTES_LEN {
            return Err(BinaryError::Truncated {
                expected: Self::BYTES_LEN,
                actual: buffer.remaining(),
            });
        }

        let len = L::decode(buffer)?;
        let mut characters = [0u8; N];
        buffer.copy_to_slice(&mut characters[..len.into_usize()]);
        Ok(Self { len, characters })
    }
}

impl<L: StringLen, const N: usize> ToBytes for AsciiString<L, N> {
    fn encode<T: bytes::BufMut>(&self, buffer: &mut T) -> Result<(), BinaryError> {
        if buffer.remaining_mut() < Self::BYTES_LEN {
            return Err(BinaryError::BufferTooSmall {
                expected: Self::BYTES_LEN,
                actual: buffer.remaining_mut(),
            });
        }

        L::encode(&self.len, buffer)?;
        buffer.put_slice(&self.characters[..self.len.into_usize()]);
        Ok(())
    }

    fn encoded_len(&self) -> usize {
        Self::BYTES_LEN
    }
}
