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

pub trait StringLen: Sized + Copy + FromBytes + ToBytes {
    fn new() -> Self;
    fn from(value: usize) -> Self;
    fn into(self) -> usize;
    fn plus_one(self) -> Self;
}

impl StringLen for u8 {
    fn new() -> Self {
        0
    }

    fn from(value: usize) -> Self {
        value as u8
    }

    fn into(self) -> usize {
        self as usize
    }

    fn plus_one(self) -> Self {
        self + 1
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

    fn from(value: usize) -> Self {
        value as u16
    }

    fn into(self) -> usize {
        self as usize
    }

    fn plus_one(self) -> Self {
        self + 1
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
    const LEN_SIZE: usize = std::mem::size_of::<L>();
    pub const MAX_LEN: usize = N;

    pub fn new(value: &str) -> Self {
        let mut characters = [0u8; N];
        let mut len = L::new();

        for (i, c) in value.to_cip_ascii_iter().enumerate() {
            if i >= Self::MAX_LEN {
                break;
            }
            characters[i] = c as u8;
            len = len.plus_one();
        }

        Self { len, characters }
    }

    pub fn from_bytes(value: &[u8]) -> Self {
        let mut characters = [0u8; N];
        let len = usize::min(value.len(), Self::MAX_LEN);
        characters[..len].copy_from_slice(&value[..len]);

        Self {
            len: L::from(len),
            characters,
        }
    }

    pub fn len(&self) -> usize {
        self.len.into()
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
        if buffer.remaining() < Self::LEN_SIZE {
            return Err(BinaryError::Truncated {
                expected: Self::LEN_SIZE,
                actual: buffer.remaining(),
            });
        }

        let len = L::decode(buffer)?;
        if len.into() > N {
            return Err(BinaryError::InvalidData {
                message: "ASCII string length exceeds max".to_string(),
                expected: N.to_string(),
                actual: len.into().to_string(),
            });
        }

        if buffer.remaining() < len.into() {
            return Err(BinaryError::Truncated {
                expected: len.into(),
                actual: buffer.remaining(),
            });
        }

        let mut characters = [0u8; N];
        buffer.copy_to_slice(&mut characters[..len.into()]);
        Ok(Self { len, characters })
    }
}

impl<L: StringLen, const N: usize> ToBytes for AsciiString<L, N> {
    fn encode<T: bytes::BufMut>(&self, buffer: &mut T) -> Result<(), BinaryError> {
        if buffer.remaining_mut() < self.encoded_len() {
            return Err(BinaryError::BufferTooSmall {
                expected: self.encoded_len(),
                actual: buffer.remaining_mut(),
            });
        }

        L::encode(&self.len, buffer)?;
        buffer.put_slice(&self.characters[..self.len.into()]);
        Ok(())
    }

    fn encoded_len(&self) -> usize {
        self.len() + Self::LEN_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::binary::BinaryError;
    use bytes::{Buf, Bytes, BytesMut};

    #[test]
    fn cip_ascii_ext_special_chars_convert_correctly() {
        assert_eq!("áéíóúç".to_cip_ascii_iter().collect::<String>(), "aeiouc");
        assert_eq!("ÁÉÍÓÚÇ".to_cip_ascii_iter().collect::<String>(), "AEIOUC");
        assert_eq!(
            "Hello!123".to_cip_ascii_iter().collect::<String>(),
            "Hello!123"
        );
        assert_eq!(
            "Hello\x01World".to_cip_ascii_iter().collect::<String>(),
            "Hello.World"
        );
    }

    #[test]
    fn string_len_u8_round_trip_success() {
        let val: u8 = 10;
        let mut buf = BytesMut::new();

        val.encode(&mut buf).expect("Failed to encode");

        assert_eq!(buf.as_ref(), &[10]);
        let mut cursor = buf.freeze();
        assert_eq!(u8::decode(&mut cursor).expect("Failed to decode"), 10);
    }

    #[test]
    fn string_len_u16_round_trip_success() {
        let val: u16 = 0x1234;
        let mut buf = BytesMut::new();

        val.encode(&mut buf).expect("Failed to encode");

        assert_eq!(buf.as_ref(), &[0x34, 0x12]);
        let mut cursor = buf.freeze();
        assert_eq!(u16::decode(&mut cursor).expect("Failed to decode"), 0x1234);
    }

    #[test]
    fn ascii_string_new_valid_content_success() {
        let s: AsciiString<u8, 5> = AsciiString::new("ABC");

        assert_eq!(s.len(), 3);
        assert_eq!(s.value(), "ABC");
        assert_eq!(s.characters[0], b'A');
        assert_eq!(s.characters[1], b'B');
        assert_eq!(s.characters[2], b'C');
    }

    #[test]
    fn ascii_string_u16_len_round_trip_success() {
        let raw_bytes: [u8; 6] = [
            0x04, 0x00, // Length: 4 (u16 LE)
            b'T', b'e', b's', b't', // Data
        ];

        let mut cursor = Bytes::copy_from_slice(&raw_bytes);
        let decoded: AsciiString<u16, 10> =
            AsciiString::decode(&mut cursor).expect("Failed to decode");

        assert_eq!(decoded.len(), 4);
        assert_eq!(decoded.value(), "Test");

        let mut buffer = BytesMut::with_capacity(decoded.encoded_len());
        decoded.encode(&mut buffer).expect("Failed to encode");
        assert_eq!(
            buffer.as_ref(),
            &raw_bytes,
            "Inconsistent encode/decode symmetry"
        );
    }

    #[test]
    fn ascii_string_empty_round_trip_success() {
        let s: AsciiString<u8, 4> = AsciiString::new("");

        let mut buf = BytesMut::with_capacity(s.encoded_len());
        s.encode(&mut buf).expect("Failed to encode");

        assert_eq!(buf.as_ref(), &[0]); // Only length byte (0)
        assert_eq!(s.len(), 0);
        assert_eq!(s.value(), "");
    }

    #[test]
    fn ascii_string_over_max_capacity_is_truncated() {
        let s: AsciiString<u8, 3> = AsciiString::new("ABCD");

        assert_eq!(s.len(), 3);
        assert_eq!(s.value(), "ABC");
    }

    #[test]
    fn ascii_string_truncated_buffer_returns_error() {
        // Length 2 but only 1 byte follows
        let raw_bytes = [0x02, b'A'];
        let mut cursor = Bytes::copy_from_slice(&raw_bytes);

        let result = AsciiString::<u8, 4>::decode(&mut cursor);

        match result {
            Err(BinaryError::Truncated { expected, actual }) => {
                assert_eq!(expected, 2);
                assert_eq!(actual, 1);
            }
            _ => panic!("Expected Truncated error, got {:?}", result),
        }
    }

    #[test]
    fn ascii_string_buffer_too_small_returns_error() {
        let s: AsciiString<u8, 10> = AsciiString::new("Hello");
        let mut raw_buf = [0u8; 2];
        let mut buf = &mut raw_buf[..]; // Fixed-size BufMut

        let result = s.encode(&mut buf);

        match result {
            Err(BinaryError::BufferTooSmall { expected, actual }) => {
                assert_eq!(expected, 6);
                assert_eq!(actual, 2);
            }
            _ => panic!("Expected BufferTooSmall error, got {:?}", result),
        }
    }

    #[test]
    fn ascii_string_fixed_buffer_serialization_only_writes_len_bytes() {
        let mut characters = [0u8; 10];
        characters[..3].copy_from_slice(b"ABC");
        // Fill rest with garbage to ensure it's not serialized
        characters[3..].fill(0xFF);
        let s: AsciiString<u8, 10> = AsciiString { len: 3, characters };

        let mut buf = BytesMut::with_capacity(20);
        s.encode(&mut buf).expect("Failed to encode");

        // Should only be 1 (len) + 3 (data) = 4 bytes
        assert_eq!(buf.len(), 4);
        assert_eq!(buf.as_ref(), &[0x03, b'A', b'B', b'C']);

        // Decode back ensuring only 3 bytes are read
        let mut cursor = Bytes::copy_from_slice(&[0x03, b'X', b'Y', b'Z', 0xFF, 0xFF]);
        let decoded: AsciiString<u8, 10> =
            AsciiString::decode(&mut cursor).expect("Failed to decode");
        assert_eq!(decoded.len(), 3);
        assert_eq!(decoded.value(), "XYZ");
        assert_eq!(cursor.remaining(), 2); // Remaining garbage should still be there
    }
}
