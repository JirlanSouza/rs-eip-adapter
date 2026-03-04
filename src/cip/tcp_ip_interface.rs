use std::net::Ipv4Addr;

use bytes::Buf;

use cip_macros::{cip_class, cip_instance, cip_object_impl};

use super::{
    ClassCode,
    common::error::CipError,
    common::object::{CipClass, CipInstance, CipObject, CipResult},
};
use crate::{
    cip::data_types::{
        CipString, UDInt,
        epath::{PortSegment, Segment},
    },
    common::binary::{FromBytes, ToBytes},
};
use crate::{
    cip::data_types::{DWord, epath::PaddedEPath},
    common::binary::BinaryError,
};

const AF_INET: u16 = 2;
pub const EIP_RESERVED_PORT: u16 = 0xAF12;

#[cip_class(id = ClassCode::TcpIpInterface, name = "TCP/IP Interface", singleton = false)]
pub struct TcpIpInterfaceClass {}

#[cip_object_impl]
impl TcpIpInterfaceClass {}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PhysicalLink {
    path_size: u16,
    path: PaddedEPath,
}

impl PhysicalLink {
    pub const MIN_LEN: usize = 4;

    pub fn new(path: PaddedEPath) -> Self {
        Self {
            path_size: (path.encoded_len() / 2) as u16,
            path,
        }
    }
}

impl FromBytes for PhysicalLink {
    fn decode<T: Buf>(buffer: &mut T) -> Result<Self, BinaryError> {
        if buffer.remaining() < Self::MIN_LEN {
            return Err(BinaryError::Truncated {
                expected: Self::MIN_LEN,
                actual: buffer.remaining(),
            });
        }

        let path_size = buffer.get_u16_le();
        let path_size_bytes = (path_size * 2) as usize;

        if buffer.remaining() < path_size_bytes {
            return Err(BinaryError::Truncated {
                expected: path_size_bytes,
                actual: buffer.remaining(),
            });
        }

        let path = PaddedEPath::decode(&mut buffer.copy_to_bytes(path_size_bytes))?;
        Ok(Self { path_size, path })
    }
}

impl ToBytes for PhysicalLink {
    fn encode<T: bytes::BufMut>(&self, buffer: &mut T) -> Result<(), BinaryError> {
        if buffer.remaining_mut() < self.encoded_len() {
            return Err(BinaryError::BufferTooSmall {
                expected: self.encoded_len(),
                actual: buffer.remaining_mut(),
            });
        }

        buffer.put_u16_le(self.path_size);
        self.path.encode(buffer)?;
        Ok(())
    }

    fn encoded_len(&self) -> usize {
        self.path.encoded_len() + 2
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterfaceConfiguration {
    ip_address: UDInt,
    network_mask: UDInt,
    gateway_address: UDInt,
    name_server: UDInt,
    name_server_2: UDInt,
    domain_name: CipString<48>,
}

impl InterfaceConfiguration {
    pub fn new() -> Self {
        Self {
            ip_address: UDInt::new(0),
            network_mask: UDInt::new(0),
            gateway_address: UDInt::new(0),
            name_server: UDInt::new(0),
            name_server_2: UDInt::new(0),
            domain_name: CipString::new(""),
        }
    }
}

impl FromBytes for InterfaceConfiguration {
    fn decode<T: Buf>(buffer: &mut T) -> Result<Self, BinaryError> {
        let ip_address = UDInt::decode(buffer)?;
        let network_mask = UDInt::decode(buffer)?;
        let gateway_address = UDInt::decode(buffer)?;
        let name_server = UDInt::decode(buffer)?;
        let name_server_2 = UDInt::decode(buffer)?;
        let domain_name = CipString::decode(buffer)?;
        Ok(Self {
            ip_address,
            network_mask,
            gateway_address,
            name_server,
            name_server_2,
            domain_name,
        })
    }
}

impl ToBytes for InterfaceConfiguration {
    fn encode<T: bytes::BufMut>(&self, buffer: &mut T) -> Result<(), BinaryError> {
        if buffer.remaining_mut() < self.encoded_len() {
            return Err(BinaryError::BufferTooSmall {
                expected: self.encoded_len(),
                actual: buffer.remaining_mut(),
            });
        }

        self.ip_address.encode(buffer)?;
        self.network_mask.encode(buffer)?;
        self.gateway_address.encode(buffer)?;
        self.name_server.encode(buffer)?;
        self.name_server_2.encode(buffer)?;
        self.domain_name.encode(buffer)?;
        Ok(())
    }

    fn encoded_len(&self) -> usize {
        return self.ip_address.encoded_len()
            + self.network_mask.encoded_len()
            + self.gateway_address.encoded_len()
            + self.name_server.encoded_len()
            + self.name_server_2.encoded_len()
            + self.domain_name.encoded_len();
    }
}

#[derive(Debug)]
#[cip_instance]
pub struct TcpIpInterfaceInstance {
    id: u16,
    class_id: ClassCode,

    #[attribute(id = 1, name = "Status", access = "get")]
    status: DWord,

    #[attribute(id = 2, name = "Configuration Capability", access = "get")]
    configuration_capability: DWord,

    #[attribute(id = 3, name = "Configuration Control", access = "get")]
    configuration_control: DWord,

    #[attribute(id = 4, name = "Physical Link Object", access = "get")]
    phisical_link_object: PhysicalLink,

    #[attribute(id = 5, name = "Interface Configuration", access = "set")]
    interface_configuration: InterfaceConfiguration,

    #[attribute(id = 6, name = "Host Name", access = "get")]
    host_name: CipString<64>,
}

#[cip_object_impl]
impl TcpIpInterfaceInstance {
    pub fn new(id: u16, address: Ipv4Addr) -> Self {
        let port_segment = PortSegment::from_port_and_ip(1, address);
        let physical_link = PhysicalLink::new(PaddedEPath::new(vec![Segment::Port(port_segment)]));

        Self {
            id,
            class_id: ClassCode::TcpIpInterface,
            status: DWord::new(0),
            configuration_capability: DWord::new(0),
            configuration_control: DWord::new(0),
            phisical_link_object: physical_link,
            interface_configuration: InterfaceConfiguration::new(),
            host_name: CipString::new(""),
        }
    }

    pub fn sin_family(&self) -> u16 {
        AF_INET
    }

    pub fn sin_addr(&self) -> [u8; 4] {
        self.interface_configuration
            .ip_address
            .value()
            .to_be_bytes()
    }

    pub fn sin_port(&self) -> u16 {
        EIP_RESERVED_PORT
    }

    pub fn sin_zero(&self) -> [u8; 8] {
        [0; 8]
    }
}
