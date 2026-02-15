use crate::encap::{
    command::EncapsulationCommand,
    error::{EncapsulationError, FrameError},
};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::io;

pub const ENCAPSULATION_HEADER_SIZE: usize = 24;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncapsulationHeader {
    pub command: EncapsulationCommand,
    pub length: u16,
    pub session_handle: u32,
    pub status: u32,
    pub context: [u8; 8],
    pub options: u32,
}

impl EncapsulationHeader {
    pub fn decode(in_buff: &mut Bytes) -> Result<Self, FrameError> {
        if in_buff.remaining() < ENCAPSULATION_HEADER_SIZE {
            return Err(FrameError::Inconplete(in_buff.len()));
        }

        let command = EncapsulationCommand::from_u16(in_buff.get_u16_le());
        let length = in_buff.get_u16_le();
        let session_handle = in_buff.get_u32_le();
        let status = in_buff.get_u32_le();
        let mut context = [0u8; 8];
        in_buff.copy_to_slice(&mut context);
        let options = in_buff.get_u32_le();
        Ok(Self {
            command,
            length,
            session_handle,
            status,
            context,
            options,
        })
    }

    pub fn encode<T: BufMut>(&self, out_buff: &mut T) -> io::Result<()> {
        if out_buff.remaining_mut() < ENCAPSULATION_HEADER_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Insufficient buffer space",
            ));
        }

        out_buff.put_u16_le(self.command.to_u16());
        out_buff.put_u16_le(self.length);
        out_buff.put_u32_le(self.session_handle);
        out_buff.put_u32_le(self.status);
        out_buff.put_slice(&self.context);
        out_buff.put_u32_le(self.options);
        Ok(())
    }

    pub fn create_error_response(
        mut header: EncapsulationHeader,
        status: EncapsulationError,
    ) -> Option<Bytes> {
        header.status = status.to_u32();
        header.length = 0;
        let mut reply_buf = BytesMut::with_capacity(ENCAPSULATION_HEADER_SIZE);
        if header.encode(&mut reply_buf).is_ok() {
            Some(reply_buf.freeze())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{BufMut, Bytes, BytesMut};
    use std::io;

    #[test]
    fn header_encode_decode_roundtrip() {
        let header = EncapsulationHeader {
            command: EncapsulationCommand::ListIdentity,
            length: 0x0010,
            session_handle: 0x11223344,
            status: 0x55667788,
            context: [1, 2, 3, 4, 5, 6, 7, 8],
            options: 0x99AABBCC,
        };

        let mut buf = BytesMut::with_capacity(ENCAPSULATION_HEADER_SIZE);
        header.encode(&mut buf).expect("encode should succeed");

        let mut bytes = buf.freeze();
        let decoded_header =
            EncapsulationHeader::decode(&mut bytes).expect("decode should succeed");
        assert_eq!(decoded_header, header);
    }

    #[test]
    fn decode_returns_incomplete_for_small_buffer() {
        let mut small = Bytes::from(vec![0u8; ENCAPSULATION_HEADER_SIZE - 1]);
        let result = EncapsulationHeader::decode(&mut small);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), FrameError::Inconplete(small.len()));
    }

    #[test]
    fn encode_fails_with_insufficient_buffer() {
        let header = EncapsulationHeader {
            command: EncapsulationCommand::Nop,
            length: 0,
            session_handle: 0,
            status: 0,
            context: [0; 8],
            options: 0,
        };

        let mut small_buf = [0u8; 8];
        let mut small_buf_slice = &mut small_buf[..];
        let encoder_result = header.encode(&mut small_buf_slice);
        assert!(encoder_result.is_err());
        if let Err(e) = encoder_result {
            assert_eq!(e.kind(), io::ErrorKind::InvalidInput);
        }
    }

    #[test]
    fn encapsulation_header_roundtrip_encode_decode_with_slice_view() {
        let header = EncapsulationHeader {
            command: EncapsulationCommand::ListIdentity,
            length: 5,
            session_handle: 0x11223344,
            status: 0x0,
            context: [1, 2, 3, 4, 5, 6, 7, 8],
            options: 0x99,
        };

        let mut buf = BytesMut::with_capacity(ENCAPSULATION_HEADER_SIZE);
        buf.put_bytes(0, ENCAPSULATION_HEADER_SIZE);
        {
            let mut buf_view = &mut buf[..ENCAPSULATION_HEADER_SIZE];
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
            status: 0,
            context: [0; 8],
            options: 0,
        };

        let mut buf = BytesMut::with_capacity(ENCAPSULATION_HEADER_SIZE);
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
            status: 0x55667788,
            context: [1, 2, 3, 4, 5, 6, 7, 8],
            options: 0x99AABBCC,
        };

        let mut buf = BytesMut::with_capacity(ENCAPSULATION_HEADER_SIZE + 4);
        header.encode(&mut buf).expect("encode should succeed");
        buf.put_u32_le(0xdeadbeef);

        let mut bytes = buf.freeze();
        let _ = EncapsulationHeader::decode(&mut bytes).expect("decode should succeed");
        assert_eq!(bytes.remaining(), 4);
        assert_eq!(bytes.get_u32_le(), 0xdeadbeef);
    }
}
