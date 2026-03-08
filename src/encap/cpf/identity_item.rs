use bytes::{Buf, BufMut};

use crate::cip::{
    cip_identity::IdentityInstance, data_types::short_string::ShortString,
    tcp_ip_interface::TcpIpInterfaceInstance,
};
use crate::common::binary::{BinaryError, FromBytes, ToBytes};
use crate::encap::cpf::cpf_item::{CpfItemDataFromBytes, CpfItemDataToBytes};

#[derive(Debug, PartialEq)]
pub struct IdentityItem {
    pub protocol_version: u16,
    pub sin_family: u16,
    pub sin_port: u16,
    pub sin_addr: u32,
    pub sin_zero: [u8; 8],
    pub vendor_id: u16,
    pub device_type: u16,
    pub product_code: u16,
    pub revision_major: u8,
    pub revision_minor: u8,
    pub status: u16,
    pub serial_number: u32,
    pub product_name: ShortString,
    pub state: u8,
}

impl IdentityItem {
    const FIXED_DATA_LEN: usize = 34;

    pub fn new(
        protocol_version: u16,
        tcp_ip_if: &TcpIpInterfaceInstance,
        identity: &IdentityInstance,
    ) -> Self {
        let interface_configuration = tcp_ip_if.get_interface_configuration();

        Self {
            protocol_version,
            sin_family: TcpIpInterfaceInstance::SIN_FAMILY,
            sin_port: TcpIpInterfaceInstance::SIN_PORT,
            sin_addr: interface_configuration.ip_address.value(),
            sin_zero: TcpIpInterfaceInstance::SIN_ZERO,
            vendor_id: identity.vendor_id,
            device_type: identity.device_type,
            product_code: identity.product_code,
            revision_major: identity.revision.major,
            revision_minor: identity.revision.minor,
            status: identity.status,
            serial_number: identity.serial_number,
            product_name: identity.product_name,
            state: identity.state.into(),
        }
    }
}

impl CpfItemDataFromBytes for IdentityItem {
    fn decode<T: Buf>(buffer: &mut T, item_len: u16) -> Result<Self, BinaryError> {
        let item_len = item_len as usize;
        if item_len < Self::FIXED_DATA_LEN {
            return Err(BinaryError::InvalidData {
                message: "Invalid IdentityItem length".to_string(),
                expected: Self::FIXED_DATA_LEN.to_string(),
                actual: item_len.to_string(),
            });
        }

        if buffer.remaining() < item_len {
            return Err(BinaryError::BufferTooSmall {
                expected: item_len,
                actual: buffer.remaining(),
            });
        }

        let mut item_buffer = buffer.take(item_len);

        let protocol_version = item_buffer.get_u16_le();
        let sin_family = item_buffer.get_u16();
        let sin_port = item_buffer.get_u16();
        let sin_addr = item_buffer.get_u32();
        let mut sin_zero = [0u8; 8];
        item_buffer.copy_to_slice(&mut sin_zero);
        let vendor_id = item_buffer.get_u16_le();
        let device_type = item_buffer.get_u16_le();
        let product_code = item_buffer.get_u16_le();
        let revision_major = item_buffer.get_u8();
        let revision_minor = item_buffer.get_u8();
        let status = item_buffer.get_u16_le();
        let serial_number = item_buffer.get_u32_le();
        let product_name = ShortString::decode(&mut item_buffer)?;
        let state = item_buffer.get_u8();

        let remaining = item_buffer.remaining();
        if remaining > 0 {
            buffer.advance(remaining);
        }

        Ok(Self {
            protocol_version,
            sin_family,
            sin_port,
            sin_addr,
            sin_zero,
            vendor_id,
            device_type,
            product_code,
            revision_major,
            revision_minor,
            status,
            serial_number,
            product_name,
            state,
        })
    }
}

impl CpfItemDataToBytes for IdentityItem {}

impl ToBytes for IdentityItem {
    fn encode<T: BufMut>(&self, buffer: &mut T) -> Result<(), BinaryError> {
        let total_len = self.encoded_len();
        if buffer.remaining_mut() < total_len {
            return Err(BinaryError::BufferTooSmall {
                expected: total_len,
                actual: buffer.remaining_mut(),
            });
        }

        buffer.put_u16_le(self.protocol_version);
        buffer.put_u16(self.sin_family);
        buffer.put_u16(self.sin_port);
        buffer.put_u32(self.sin_addr);
        buffer.put_slice(&self.sin_zero);
        buffer.put_u16_le(self.vendor_id);
        buffer.put_u16_le(self.device_type);
        buffer.put_u16_le(self.product_code);
        buffer.put_u8(self.revision_major);
        buffer.put_u8(self.revision_minor);
        buffer.put_u16_le(self.status);
        buffer.put_u32_le(self.serial_number);
        self.product_name.encode(buffer)?;
        buffer.put_u8(self.state);
        Ok(())
    }

    fn encoded_len(&self) -> usize {
        Self::FIXED_DATA_LEN + self.product_name.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{BufMut, Bytes, BytesMut};

    #[test]
    fn identity_item_encode_decode_symmetry() {
        let mut raw_bytes = Vec::new();
        raw_bytes.extend_from_slice(&1u16.to_le_bytes()); // protocol_version
        raw_bytes.extend_from_slice(&2u16.to_be_bytes()); // sin_family
        raw_bytes.extend_from_slice(&44818u16.to_be_bytes()); // sin_port
        raw_bytes.extend_from_slice(&0x7F000001u32.to_be_bytes()); // sin_addr (127.0.0.1)
        raw_bytes.extend_from_slice(&[0u8; 8]); // sin_zero
        raw_bytes.extend_from_slice(&1234u16.to_le_bytes()); // vendor_id
        raw_bytes.extend_from_slice(&7u16.to_le_bytes()); // device_type
        raw_bytes.extend_from_slice(&5678u16.to_le_bytes()); // product_code
        long_name_helper(&mut raw_bytes);
    }

    fn long_name_helper(raw_bytes: &mut Vec<u8>) {
        raw_bytes.push(1); // revision_major
        raw_bytes.push(2); // revision_minor
        raw_bytes.extend_from_slice(&0x0001u16.to_le_bytes()); // status
        raw_bytes.extend_from_slice(&0x12345678u32.to_le_bytes()); // serial_number
        raw_bytes.push(4); // name len
        raw_bytes.extend_from_slice(b"Test"); // product_name
        raw_bytes.push(3); // state (Operational)

        let mut cursor = Bytes::copy_from_slice(raw_bytes);
        let decoded =
            IdentityItem::decode(&mut cursor, raw_bytes.len() as u16).expect("Failed to decode");

        assert_eq!(decoded.protocol_version, 1);
        assert_eq!(decoded.sin_family, 2);
        assert_eq!(decoded.sin_port, 44818);
        assert_eq!(decoded.sin_addr, 0x7F000001);
        assert_eq!(decoded.vendor_id, 1234);
        assert_eq!(decoded.product_name.value(), "Test");
        assert_eq!(decoded.state, 3);

        let mut buffer = BytesMut::with_capacity(decoded.encoded_len());
        decoded.encode(&mut buffer).expect("Failed to encode");
        assert_eq!(
            buffer.as_ref(),
            raw_bytes.as_slice(),
            "Inconsistent encode/decode symmetry"
        );
    }

    #[test]
    fn identity_item_decode_invalid_length_fails() {
        let mut buffer = BytesMut::new();
        buffer.put_u16_le(1);
        let mut cursor = buffer.freeze();
        let result = IdentityItem::decode(&mut cursor, 33);
        assert!(result.is_err());
    }

    #[test]
    fn identity_item_decode_buffer_too_small_fails() {
        let mut buffer = BytesMut::new();
        buffer.put_u16_le(1);
        let mut cursor = buffer.freeze();
        let result = IdentityItem::decode(&mut cursor, 40);
        assert!(result.is_err());
    }

    #[test]
    fn identity_item_encode_buffer_too_small_fails() {
        let item = IdentityItem {
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
            product_name: ShortString::from("Name"),
            state: 12,
        };

        let mut data = [0u8; 10];
        let mut buffer = &mut data[..];
        let result = item.encode(&mut buffer);
        assert!(result.is_err());
    }

    #[test]
    fn identity_item_with_long_name_success() {
        let long_name = "A".repeat(255);
        let item = IdentityItem {
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
            product_name: ShortString::from(long_name.as_str()),
            state: 12,
        };

        let mut buffer = BytesMut::with_capacity(item.encoded_len());
        item.encode(&mut buffer).expect("Failed to encode");

        let mut cursor = buffer.freeze();
        let decoded =
            IdentityItem::decode(&mut cursor, item.encoded_len() as u16).expect("Failed to decode");
        assert_eq!(decoded.product_name.value(), long_name);
    }
}
