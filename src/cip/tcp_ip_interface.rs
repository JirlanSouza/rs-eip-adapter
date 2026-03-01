use std::net::Ipv4Addr;

use cip_macros::{cip_class, cip_instance, cip_object_impl};

use super::{
    ClassCode,
    common::error::CipError,
    common::object::{CipClass, CipInstance, CipObject, CipResult},
};

const AF_INET: u16 = 2;
pub const EIP_RESERVED_PORT: u16 = 0xAF12;

#[cip_class(id = ClassCode::TcpIpInterface, name = "TCP/IP Interface", singleton = false)]
pub struct TcpIpInterfaceClass {}

#[cip_object_impl]
impl TcpIpInterfaceClass {}

#[derive(Debug)]
#[cip_instance]
pub struct TcpIpInterfaceInstance {
    id: u16,
    class_id: ClassCode,
    address: Ipv4Addr,
}

#[cip_object_impl]
impl TcpIpInterfaceInstance {
    pub fn new(id: u16, address: Ipv4Addr) -> Self {
        Self {
            id,
            class_id: ClassCode::TcpIpInterface,
            address,
        }
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
