use std::fmt::Display;

pub mod cip_class;
pub mod cip_error;
pub mod cip_identity;
pub mod registry;
pub mod tcp_ip_interface;

#[repr(u16)]
pub enum CipClassId {
    IdentityClassId = 0x01,
    TcpIpInterfaceClassId = 0xF5,
    UserDefinedClassId(u16),
}

impl CipClassId {
    pub fn from_u16(id: u16) -> Option<Self> {
        match id {
            0x01 => Some(CipClassId::IdentityClassId),
            0x02 => Some(CipClassId::TcpIpInterfaceClassId),
            _ => Some(CipClassId::UserDefinedClassId(id)),
        }
    }

    pub fn to_u16(&self) -> u16 {
        match self {
            CipClassId::IdentityClassId => 0x01,
            CipClassId::TcpIpInterfaceClassId => 0x02,
            CipClassId::UserDefinedClassId(id) => *id,
        }
    }
}

impl Display for CipClassId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CipClassId::IdentityClassId => write!(f, "{}: Identity Class", self.to_u16()),
            CipClassId::TcpIpInterfaceClassId => {
                write!(f, "{}: TCP/IP Interface Class", self.to_u16())
            }
            CipClassId::UserDefinedClassId(id) => write!(f, "{}: User Defined Class", id),
        }
    }
}
