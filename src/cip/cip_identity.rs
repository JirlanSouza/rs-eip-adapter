use std::{
    any::Any,
    sync::{Arc, RwLock, Weak},
};

pub struct IdentityInfo {
    pub vendor_id: u16,
    pub device_type: u16,
    pub product_code: u16,
    pub revision_major: u8,
    pub revision_minor: u8,
    pub serial_number: u32,
    pub product_name: &'static str,
}

use crate::cip::{
    CipClassId,
    cip_class::{CipClass, CipInstance},
    cip_error::CipError,
};

pub struct IdentityClass {
    class_id: u16,
    class_name: &'static str,
    instance: RwLock<Arc<IdentityInstance>>,
}

impl IdentityClass {
    pub fn new(info: &IdentityInfo) -> Arc<Self> {
        Arc::new_cyclic(|class_weak| {
            let inst = IdentityInstance::new(
                class_weak.clone() as Weak<dyn CipClass>,
                info.vendor_id,
                info.device_type,
                info.product_code,
                info.revision_major,
                info.revision_minor,
                0,
                info.serial_number,
                info.product_name,
                0,
            );

            Self {
                class_id: CipClassId::IdentityClassId.to_u16(),
                class_name: "Identity",
                instance: RwLock::new(Arc::new(inst)),
            }
        })
    }
}

impl CipClass for IdentityClass {
    fn class_id(&self) -> u16 {
        self.class_id
    }

    fn class_name(&self) -> &'static str {
        self.class_name
    }

    fn instance(&self, instance_id: u16) -> Result<Arc<dyn CipInstance>, CipError> {
        if instance_id != 1 {
            return Err(CipError::ObjectDoesNotExist);
        }

        let read_guard = self
            .instance
            .read()
            .map_err(|_| {
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
    pub revision_major: u8,
    pub revision_minor: u8,
    pub status: u16,
    pub serial_number: u32,
    pub product_name: String,
    pub state: u8,
}

impl IdentityInstance {
    pub fn new(
        class: Weak<dyn CipClass>,
        vendor_id: u16,
        device_type: u16,
        product_code: u16,
        revision_major: u8,
        revision_minor: u8,
        status: u16,
        serial_number: u32,
        product_name: impl Into<String>,
        state: u8,
    ) -> Self {
        Self {
            class,
            vendor_id,
            device_type,
            product_code,
            revision_major,
            revision_minor,
            status,
            serial_number,
            product_name: product_name.into(),
            state,
        }
    }
}

impl CipInstance for IdentityInstance {
    fn class(&self) -> Weak<dyn CipClass> {
        self.class.clone()
    }

    fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}
