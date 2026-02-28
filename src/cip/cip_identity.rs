use std::{
    any::Any,
    sync::{Arc, RwLock, Weak},
};

use bytes::BufMut;

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

pub struct IdentityClass {
    instance: RwLock<Arc<IdentityInstance>>,
}

impl IdentityClass {
    pub fn new(info: &IdentityInfo) -> Arc<Self> {
        Arc::new_cyclic(|class_weak| {
            let inst = IdentityInstance::new(class_weak.clone() as Weak<dyn CipClass>, info);

            Self {
                instance: RwLock::new(Arc::new(inst)),
            }
        })
    }
}

impl CipObject for IdentityClass {
    fn execute_service(
        &self,
        _service_id: u8,
        _req: bytes::Bytes,
        _resp: &mut bytes::BytesMut,
    ) -> CipResult {
        Err(CipError::ServiceNotSupported)
    }
}

impl CipClass for IdentityClass {
    fn class_id(&self) -> u16 {
        ClassCode::IdentityClassId.into()
    }

    fn class_name(&self) -> &'static str {
        "Identity"
    }
    fn instance(&self, instance_id: u16) -> Result<Arc<dyn CipInstance>, CipError> {
        if instance_id != 1 {
            return Err(CipError::ObjectDoesNotExist);
        }

        let read_guard = self.instance.read().map_err(|_| {
            log::error!("Failed to get read guard for IdentityClass instance");
            CipError::GeneralError
        })?;

        let inst = Arc::clone(&read_guard);
        Ok(inst as Arc<dyn CipInstance>)
    }

    fn add_instance(&self, _instance: Arc<dyn CipInstance>) -> Result<(), CipError> {
        Err(CipError::ObjectStateConflict)
    }
}

#[derive(Debug)]
pub struct IdentityInstance {
    class: Weak<dyn CipClass>,
    pub vendor_id: u16,
    pub device_type: u16,
    pub product_code: u16,
    pub revision: Revision,
    pub status: u16,
    pub serial_number: u32,
    pub product_name: ShortString,
    pub state: DeviceState,
}

impl IdentityInstance {
    const BASE_ATTRIBUTES_LEN: usize = 15;

    pub fn new(class: Weak<dyn CipClass>, info: &IdentityInfo) -> Self {
        Self {
            class,
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

    pub fn get_attribute_all(&self, _req: bytes::Bytes, resp: &mut bytes::BytesMut) -> CipResult {
        self.encode(resp)?;
        Ok(())
    }
}

impl CipInstance for IdentityInstance {
    fn instance_id(&self) -> u16 {
        1
    }

    fn class(&self) -> Weak<dyn CipClass> {
        self.class.clone()
    }

    fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

impl CipObject for IdentityInstance {
    fn execute_service(
        &self,
        service_id: u8,
        req: bytes::Bytes,
        resp: &mut bytes::BytesMut,
    ) -> CipResult {
        match service_id {
            1 => self.get_attribute_all(req, resp),
            _ => Err(CipError::ServiceNotSupported),
        }
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
