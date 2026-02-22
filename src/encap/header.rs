use bytes::{Buf, BufMut, BytesMut};

use crate::{
    common::binary::{BinaryError, FromBytes, ToBytes},
    encap::{command::EncapsulationCommand, error::EncapsulationError},
};

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EncapsulationStatus {
    Success = 0x0000,
    InvalidOrUnsupportedCommand = 0x0001,
    InsufficientMemory = 0x0002,
    IncorrectData = 0x0003,
    InvalidSessionHandle = 0x0064,
    InvalidLength = 0x0065,
    UnsupportedProtocol = 0x0069,
}

impl TryFrom<u32> for EncapsulationStatus {
    type Error = BinaryError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x0000 => Ok(Self::Success),
            0x0001 => Ok(Self::InvalidOrUnsupportedCommand),
            0x0002 => Ok(Self::InsufficientMemory),
            0x0003 => Ok(Self::IncorrectData),
            0x0064 => Ok(Self::InvalidSessionHandle),
            0x0065 => Ok(Self::InvalidLength),
            0x0069 => Ok(Self::UnsupportedProtocol),
            _ => Err(BinaryError::InvalidData {
                message: "Invalid encapsulation status".to_string(),
                expected: "Valid encapsulation status".to_string(),
                actual: format!("{:#x}", value),
            }),
        }
    }
}

impl From<EncapsulationStatus> for u32 {
    fn from(value: EncapsulationStatus) -> Self {
        match value {
            EncapsulationStatus::Success => 0x0000,
            EncapsulationStatus::InvalidOrUnsupportedCommand => 0x0001,
            EncapsulationStatus::InsufficientMemory => 0x0002,
            EncapsulationStatus::IncorrectData => 0x0003,
            EncapsulationStatus::InvalidSessionHandle => 0x0064,
            EncapsulationStatus::InvalidLength => 0x0065,
            EncapsulationStatus::UnsupportedProtocol => 0x0069,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EncapsulationHeader {
    pub command: EncapsulationCommand,
    pub length: u16,
    pub session_handle: u32,
    pub status: EncapsulationStatus,
    pub context: [u8; 8],
    pub options: u32,
}

impl EncapsulationHeader {
    pub const LEN: usize = 24;

    pub fn length_from_bytes(buffer: &BytesMut) -> Option<u16> {
        if buffer.remaining() < 4 {
            return None;
        }

        Some(u16::from_le_bytes([buffer[2], buffer[3]]))
    }

    pub fn clone_with_session_handle(&self, session_handle: u32) -> Self {
        Self {
            session_handle,
            ..self.clone()
        }
    }

    pub fn clone_with_error_and_length(&self, error: EncapsulationError, length: u16) -> Self {
        Self {
            status: error.into(),
            length,
            ..self.clone()
        }
    }
}

impl FromBytes for EncapsulationHeader {
    fn decode<T: Buf>(buffer: &mut T) -> Result<Self, BinaryError> {
        if buffer.remaining() < Self::LEN {
            return Err(BinaryError::Truncated {
                expected: Self::LEN,
                actual: buffer.remaining(),
            });
        }

        let command: EncapsulationCommand = buffer.get_u16_le().into();
        let length = buffer.get_u16_le();
        let session_handle = buffer.get_u32_le();
        let status = EncapsulationStatus::try_from(buffer.get_u32_le())?;
        let mut context = [0u8; 8];
        buffer.copy_to_slice(&mut context);
        let options = buffer.get_u32_le();
        Ok(Self {
            command,
            length,
            session_handle,
            status,
            context,
            options,
        })
    }
}

impl ToBytes for EncapsulationHeader {
    fn encode<T: BufMut>(&self, buffer: &mut T) -> Result<(), BinaryError> {
        if buffer.remaining_mut() < Self::LEN {
            return Err(BinaryError::BufferTooSmall {
                expected: Self::LEN,
                actual: buffer.remaining_mut(),
            });
        }

        buffer.put_u16_le(self.command.into());
        buffer.put_u16_le(self.length);
        buffer.put_u32_le(self.session_handle);
        buffer.put_u32_le(self.status.into());
        buffer.put_slice(&self.context);
        buffer.put_u32_le(self.options);
        Ok(())
    }

    fn encoded_len(&self) -> usize {
        Self::LEN
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{BufMut, Bytes, BytesMut};

    #[test]
    fn header_encode_decode_roundtrip() {
        let header = EncapsulationHeader {
            command: EncapsulationCommand::ListIdentity,
            length: 0x0010,
            session_handle: 0x11223344,
            status: EncapsulationStatus::Success,
            context: [1, 2, 3, 4, 5, 6, 7, 8],
            options: 0x99AABBCC,
        };

        let mut buf = BytesMut::with_capacity(EncapsulationHeader::LEN);
        header.encode(&mut buf).expect("encode should succeed");

        let mut bytes = buf.freeze();
        let decoded_header =
            EncapsulationHeader::decode(&mut bytes).expect("decode should succeed");
        assert_eq!(decoded_header, header);
    }

    #[test]
    fn decode_returns_incomplete_for_small_buffer() {
        let mut small = Bytes::from(vec![0u8; EncapsulationHeader::LEN - 1]);

        let result = EncapsulationHeader::decode(&mut small);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            BinaryError::Truncated {
                expected: EncapsulationHeader::LEN,
                actual: small.len()
            }
        );
    }

    #[test]
    fn encode_fails_with_insufficient_buffer() {
        let header = EncapsulationHeader {
            command: EncapsulationCommand::Nop,
            length: 0,
            session_handle: 0,
            status: EncapsulationStatus::Success,
            context: [0; 8],
            options: 0,
        };

        let mut small_buf = [0u8; 8];
        let mut small_buf_slice = &mut small_buf[..];

        let encoder_result = header.encode(&mut small_buf_slice);

        assert!(encoder_result.is_err());

        if let Err(e) = encoder_result {
            assert_eq!(
                e,
                BinaryError::BufferTooSmall {
                    expected: EncapsulationHeader::LEN,
                    actual: small_buf_slice.len()
                }
            );
        }
    }

    #[test]
    fn encapsulation_header_roundtrip_encode_decode_with_slice_view() {
        let header = EncapsulationHeader {
            command: EncapsulationCommand::ListIdentity,
            length: 5,
            session_handle: 0x11223344,
            status: EncapsulationStatus::Success,
            context: [1, 2, 3, 4, 5, 6, 7, 8],
            options: 0x99,
        };

        let mut buf = BytesMut::with_capacity(EncapsulationHeader::LEN);
        buf.put_bytes(0, EncapsulationHeader::LEN);
        {
            let mut buf_view = &mut buf[..EncapsulationHeader::LEN];
            header.encode(&mut buf_view).expect("encode should succeed");
        }

        let mut bytes = Bytes::copy_from_slice(&buf[..]);
        let decoded = EncapsulationHeader::decode(&mut bytes).expect("decode should succeed");
        assert_eq!(decoded, header);
    }

    #[test]
    fn header_with_unknown_command_roundtrip() {
        let header = EncapsulationHeader {
            command: EncapsulationCommand::Unknown(0xbeef),
            length: 0,
            session_handle: 0,
            status: EncapsulationStatus::Success,
            context: [0; 8],
            options: 0,
        };

        let mut buf = BytesMut::with_capacity(EncapsulationHeader::LEN);
        header.encode(&mut buf).expect("encode should succeed");

        let mut bytes = buf.freeze();
        let decoded_header =
            EncapsulationHeader::decode(&mut bytes).expect("decode should succeed");
        assert_eq!(decoded_header, header);
    }

    #[test]
    fn decode_does_not_consume_extra_bytes() {
        let header = EncapsulationHeader {
            command: EncapsulationCommand::ListIdentity,
            length: 0x0010,
            session_handle: 0x11223344,
            status: EncapsulationStatus::Success,
            context: [1, 2, 3, 4, 5, 6, 7, 8],
            options: 0x99AABBCC,
        };

        let mut buf = BytesMut::with_capacity(EncapsulationHeader::LEN + 4);
        header.encode(&mut buf).expect("encode should succeed");
        buf.put_u32_le(0xdeadbeef);

        let mut bytes = buf.freeze();
        let _ = EncapsulationHeader::decode(&mut bytes).expect("decode should succeed");
        assert_eq!(bytes.remaining(), 4);
        assert_eq!(bytes.get_u32_le(), 0xdeadbeef);
    }
}
