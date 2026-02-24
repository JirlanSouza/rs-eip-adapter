pub mod list_identity;
pub mod register_session;

pub use list_identity::ListIdentityHandler;
pub use register_session::{RegisterSessionData, RegisterSessionHandler};

#[repr(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EncapsulationCommand {
    Nop,
    ListServices,
    ListIdentity,
    ListInterfaces,
    RegisterSession,
    UnregisterSession,
    SendRRData,
    SendUnitData,
    IndicateStatus,
    Cancel,
    Unknown(u16),
}

impl From<u16> for EncapsulationCommand {
    fn from(value: u16) -> Self {
        match value {
            0x0000 => EncapsulationCommand::Nop,
            0x0004 => EncapsulationCommand::ListServices,
            0x0063 => EncapsulationCommand::ListIdentity,
            0x0064 => EncapsulationCommand::ListInterfaces,
            0x0065 => EncapsulationCommand::RegisterSession,
            0x0066 => EncapsulationCommand::UnregisterSession,
            0x006F => EncapsulationCommand::SendRRData,
            0x0070 => EncapsulationCommand::SendUnitData,
            0x0072 => EncapsulationCommand::IndicateStatus,
            0x0073 => EncapsulationCommand::Cancel,
            other => EncapsulationCommand::Unknown(other),
        }
    }
}

impl From<EncapsulationCommand> for u16 {
    fn from(command: EncapsulationCommand) -> u16 {
        match command {
            EncapsulationCommand::Nop => 0x0000,
            EncapsulationCommand::ListServices => 0x0004,
            EncapsulationCommand::ListIdentity => 0x0063,
            EncapsulationCommand::ListInterfaces => 0x0064,
            EncapsulationCommand::RegisterSession => 0x0065,
            EncapsulationCommand::UnregisterSession => 0x0066,
            EncapsulationCommand::SendRRData => 0x006F,
            EncapsulationCommand::SendUnitData => 0x0070,
            EncapsulationCommand::IndicateStatus => 0x0072,
            EncapsulationCommand::Cancel => 0x0073,
            EncapsulationCommand::Unknown(v) => v,
        }
    }
}
