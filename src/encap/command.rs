pub mod list_identity;
pub mod register_session;

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
    Unknown(u16),
}

impl From<u16> for EncapsulationCommand {
    fn from(v: u16) -> Self {
        match v {
            0x0000 => EncapsulationCommand::Nop,
            0x0004 => EncapsulationCommand::ListServices,
            0x0063 => EncapsulationCommand::ListIdentity,
            0x0064 => EncapsulationCommand::ListInterfaces,
            0x0065 => EncapsulationCommand::RegisterSession,
            0x0066 => EncapsulationCommand::UnregisterSession,
            0x006F => EncapsulationCommand::SendRRData,
            0x0070 => EncapsulationCommand::SendUnitData,
            other => EncapsulationCommand::Unknown(other),
        }
    }
}

impl From<EncapsulationCommand> for u16 {
    fn from(cmd: EncapsulationCommand) -> u16 {
        match cmd {
            EncapsulationCommand::Nop => 0x0000,
            EncapsulationCommand::ListServices => 0x0004,
            EncapsulationCommand::ListIdentity => 0x0063,
            EncapsulationCommand::ListInterfaces => 0x0064,
            EncapsulationCommand::RegisterSession => 0x0065,
            EncapsulationCommand::UnregisterSession => 0x0066,
            EncapsulationCommand::SendRRData => 0x006F,
            EncapsulationCommand::SendUnitData => 0x0070,
            EncapsulationCommand::Unknown(v) => v,
        }
    }
}
