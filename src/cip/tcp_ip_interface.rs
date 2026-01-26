use std::{
    any::Any,
    net::Ipv4Addr,
    sync::{Arc, RwLock, Weak},
};

use crate::cip::{
    CipClassId,
    cip_class::{CipClass, CipInstance},
    cip_error::CipError,
};

const AF_INET: u16 = 2;
pub const EIP_RESERVED_PORT: u16 = 0xAF12;

pub struct TcpIpInterfaceClass {
    class_id: u16,
    class_name: &'static str,
    instances: RwLock<Vec<Arc<dyn CipInstance>>>,
}

impl TcpIpInterfaceClass {
    pub fn new() -> Self {
        Self {
            class_id: CipClassId::TcpIpInterfaceClassId.to_u16(),
            class_name: "TcpIpInterfaceClass",
            instances: RwLock::new(Vec::with_capacity(2)),
        }
    }
}

impl CipClass for TcpIpInterfaceClass {
    fn class_id(&self) -> u16 {
        self.class_id
    }

    fn class_name(&self) -> &'static str {
        self.class_name
    }

    fn instance(&self, instance_id: u16) -> Result<Arc<dyn CipInstance>, CipError> {
        if instance_id == 0 {
            return Err(CipError::GeneralError);
        }
        
        self.instances
            .read()
            .map_err(|_| {
                log::error!("Failed to get read guard for TcpIpInterface instance: {}", instance_id);
                CipError::GeneralError
            })?
            .get((instance_id - 1) as usize)
            .cloned()
            .map(|ins| ins as Arc<dyn CipInstance>)
            .ok_or(CipError::ObjectDoesNotExist)
    }

    fn add_instance(&self, instance: Arc<dyn CipInstance>) -> Result<(), CipError> {
        self.instances
            .write()
            .map_err(|_| {
                log::error!("Failed to get write guard for TcpIpInterface instances vector");
                CipError::GeneralError
            })?
            .push(instance);

        Ok(())
    }
}

pub struct TcpIpInterfaceInstance {
    class: Weak<dyn CipClass>,
    address: Ipv4Addr,
}

impl TcpIpInterfaceInstance {
    pub fn new(class: Weak<dyn CipClass>, address: Ipv4Addr) -> Self {
        Self { class, address }
    }

    pub fn as_any(&self) -> &dyn Any {
        self
    }

    pub fn sin_family(&self) -> u16 {
        AF_INET
    }

    pub fn sin_addr(&self) -> [u8; 4] {
        self.address.octets()
    }

    pub fn sin_port(&self) -> u16 {
        EIP_RESERVED_PORT
    }

    pub fn sin_zero(&self) -> [u8; 8] {
        [0; 8]
    }
}

impl CipInstance for TcpIpInterfaceInstance {
    fn class(&self) -> Weak<dyn CipClass> {
        self.class.clone()
    }

    fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}
