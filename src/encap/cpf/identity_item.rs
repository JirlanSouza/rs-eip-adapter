use bytes::{Buf, BufMut};

use crate::cip::{
    cip_identity::IdentityInstance, data_types::short_string::ShortString,
    tcp_ip_interface::TcpIpInterfaceInstance,
};
use crate::common::binary::{BinaryError, FromBytes, ToBytes};
use crate::encap::cpf::cpf_item::{CpfItemDataFromBytes, CpfItemDataToBytes};

#[derive(Debug)]
pub struct IdentityItem {
    pub protocol_version: u16,
    pub sin_family: u16,
    pub sin_port: u16,
    pub sin_addr: [u8; 4],
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
        Self {
            protocol_version,
            sin_family: tcp_ip_if.sin_family(),
            sin_port: tcp_ip_if.sin_port(),
            sin_addr: tcp_ip_if.sin_addr(),
            sin_zero: tcp_ip_if.sin_zero(),
            vendor_id: identity.vendor_id,
            device_type: identity.device_type,
            product_code: identity.product_code,
            revision_major: identity.revision_major,
            revision_minor: identity.revision_minor,
            status: identity.status,
            serial_number: identity.serial_number,
            product_name: identity.product_name,
            state: identity.state,
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
        let mut sin_addr = [0u8; 4];
        item_buffer.copy_to_slice(&mut sin_addr);
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
        buffer.put_slice(&self.sin_addr);
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
        Self::FIXED_DATA_LEN + self.product_name.encoded_len()
    }
}
