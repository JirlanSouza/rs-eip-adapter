use std::fmt::Display;

pub mod cip_identity;
pub mod common;
pub mod data_types;
pub mod registry;
pub mod tcp_ip_interface;

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassCode {
    IdentityClassId = 0x01,
    TcpIpInterfaceClassId = 0xF5,
    UserDefinedClassId(u16),
}

impl From<u16> for ClassCode {
    fn from(id: u16) -> Self {
        match id {
            0x01 => ClassCode::IdentityClassId,
            0xF5 => ClassCode::TcpIpInterfaceClassId,
            _ => ClassCode::UserDefinedClassId(id),
        }
    }
}

impl From<&ClassCode> for u16 {
    fn from(id: &ClassCode) -> Self {
        match id {
            ClassCode::IdentityClassId => 0x01,
            ClassCode::TcpIpInterfaceClassId => 0xF5,
            ClassCode::UserDefinedClassId(id) => *id,
        }
    }
}

impl From<ClassCode> for u16 {
    fn from(id: ClassCode) -> Self {
        match id {
            ClassCode::IdentityClassId => 0x01,
            ClassCode::TcpIpInterfaceClassId => 0xF5,
            ClassCode::UserDefinedClassId(id) => id,
        }
    }
}

impl Display for ClassCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClassCode::IdentityClassId => write!(f, "{:#04x}: Identity", u16::from(self)),
            ClassCode::TcpIpInterfaceClassId => {
                write!(f, "{:#04x}: TCP/IP Interface", u16::from(self))
            }
            ClassCode::UserDefinedClassId(id) => write!(f, "{:#04x}: User Defined", id),
        }
    }
}
