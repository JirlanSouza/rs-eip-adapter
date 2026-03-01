use std::fmt::Display;

pub mod error;
pub mod object;

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassCode {
    Identity = 0x01,
    TcpIpInterface = 0xF5,
    UserDefined(u16),
}

impl From<u16> for ClassCode {
    fn from(id: u16) -> Self {
        match id {
            0x01 => ClassCode::Identity,
            0xF5 => ClassCode::TcpIpInterface,
            _ => ClassCode::UserDefined(id),
        }
    }
}

impl From<&ClassCode> for u16 {
    fn from(id: &ClassCode) -> Self {
        match id {
            ClassCode::Identity => 0x01,
            ClassCode::TcpIpInterface => 0xF5,
            ClassCode::UserDefined(id) => *id,
        }
    }
}

impl From<ClassCode> for u16 {
    fn from(id: ClassCode) -> Self {
        match id {
            ClassCode::Identity => 0x01,
            ClassCode::TcpIpInterface => 0xF5,
            ClassCode::UserDefined(id) => id,
        }
    }
}

impl Display for ClassCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClassCode::Identity => write!(f, "{:#04x}: Identity", u16::from(self)),
            ClassCode::TcpIpInterface => {
                write!(f, "{:#04x}: TCP/IP Interface", u16::from(self))
            }
            ClassCode::UserDefined(id) => write!(f, "{:#04x}: User Defined", id),
        }
    }
}
