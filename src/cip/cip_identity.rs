use std::sync::{Arc, RwLock};

use bytes::{Buf, BufMut};
use cip_macros::{cip_class, cip_instance, cip_object_impl};

use super::{
    ClassCode,
    common::error::CipError,
    common::object::{CipClass, CipInstance, CipObject, CipResult},
    data_types::short_string::ShortString,
};
use crate::common::binary::{BinaryError, FromBytes, ToBytes};

#[derive(Debug)]
pub struct IdentityInfo {
    pub vendor_id: u16,
    pub device_type: u16,
    pub product_code: u16,
    pub revision_major: u8,
    pub revision_minor: u8,
    pub serial_number: u32,
    pub product_name: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Revision {
    pub major: u8,
    pub minor: u8,
}

impl FromBytes for Revision {
    fn decode<T: bytes::Buf>(buffer: &mut T) -> Result<Self, BinaryError> {
        if buffer.remaining() < 2 {
            return Err(BinaryError::Truncated {
                expected: 2,
                actual: buffer.remaining(),
            });
        }

        let major = buffer.get_u8();
        let minor = buffer.get_u8();
        Ok(Self { major, minor })
    }
}

impl ToBytes for Revision {
    fn encode<T: BufMut>(&self, buffer: &mut T) -> Result<(), BinaryError> {
        if buffer.remaining_mut() < 2 {
            return Err(BinaryError::BufferTooSmall {
                expected: 2,
                actual: buffer.remaining_mut(),
            });
        }

        buffer.put_u8(self.major);
        buffer.put_u8(self.minor);
        Ok(())
    }

    fn encoded_len(&self) -> usize {
        2
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceState {
    Nonexistent = 0,
    DeviceSelfTesting = 1,
    Standby = 2,
    Operational = 3,
    MajorRecoverableFault = 4,
    MajorUnrecoverableFault = 5,
    Reserved = 6,
    Default = 255,
}

impl From<u8> for DeviceState {
    fn from(id: u8) -> Self {
        match id {
            0 => DeviceState::Nonexistent,
            1 => DeviceState::DeviceSelfTesting,
            2 => DeviceState::Standby,
            3 => DeviceState::Operational,
            4 => DeviceState::MajorRecoverableFault,
            5 => DeviceState::MajorUnrecoverableFault,
            6 => DeviceState::Reserved,
            255 => DeviceState::Default,
            _ => DeviceState::Default,
        }
    }
}

impl From<DeviceState> for u8 {
    fn from(state: DeviceState) -> Self {
        state as u8
    }
}

#[derive(Debug)]
pub enum Test {
    Id = 0x01,
}

impl From<Test> for u16 {
    fn from(test: Test) -> Self {
        test as u16
    }
}

#[cip_class(id = ClassCode::Identity, name = "Identity", singleton = true)]
pub struct IdentityClass {}

#[cip_object_impl]
impl IdentityClass {
    pub fn with_default_instance(info: &IdentityInfo) -> Arc<Self> {
        let instance = Arc::new(IdentityInstance::new(info));

        Arc::new(Self {
            instance: RwLock::new(instance),
        })
    }
}

#[cip_instance]
#[derive(Debug)]
pub struct IdentityInstance {
    id: u16,
    class_id: ClassCode,
    pub vendor_id: u16,
    pub device_type: u16,
    pub product_code: u16,
    pub revision: Revision,
    pub status: u16,
    pub serial_number: u32,
    pub product_name: ShortString,
    pub state: DeviceState,
}

#[cip_object_impl]
impl IdentityInstance {
    const BASE_ATTRIBUTES_LEN: usize = 15;

    pub fn new(info: &IdentityInfo) -> Self {
        Self {
            id: 1,
            class_id: ClassCode::Identity,
            vendor_id: info.vendor_id,
            device_type: info.device_type,
            product_code: info.product_code,
            revision: Revision {
                major: info.revision_major,
                minor: info.revision_minor,
            },
            status: 0,
            serial_number: info.serial_number,
            product_name: info.product_name.into(),
            state: DeviceState::Default,
        }
    }

    #[service(0x01)]
    pub fn get_attribute_all(
        &self,
        _req: &mut bytes::Bytes,
        resp: &mut bytes::BytesMut,
    ) -> CipResult {
        self.encode(resp)?;
        Ok(())
    }
}

impl ToBytes for IdentityInstance {
    fn encode<T: BufMut>(&self, buffer: &mut T) -> Result<(), BinaryError> {
        let len = self.encoded_len();
        if buffer.remaining_mut() < len {
            return Err(BinaryError::BufferTooSmall {
                expected: len,
                actual: buffer.remaining_mut(),
            });
        }

        buffer.put_u16_le(self.vendor_id);
        buffer.put_u16_le(self.device_type);
        buffer.put_u16_le(self.product_code);
        self.revision.encode(buffer)?;
        buffer.put_u16_le(self.status);
        buffer.put_u32_le(self.serial_number);
        self.product_name.encode(buffer)?;
        buffer.put_u8(self.state.into());

        Ok(())
    }

    fn encoded_len(&self) -> usize {
        Self::BASE_ATTRIBUTES_LEN + self.product_name.encoded_len()
    }
}
