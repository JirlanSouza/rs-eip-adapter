use bytes::{Buf, BufMut};

use crate::{
    common::binary::{BinaryError, FromBytes, ToBytes},
    encap::{
        command::{EncapsulationCommand, register_session::RegisterSessionData},
        cpf::Cpf,
    },
};

#[derive(Debug)]
pub enum EncapsulationPayload {
    None,
    RegisterSession(RegisterSessionData),
    Cpf(Cpf),
}

pub trait EncapsulationPayloadFromBytes: Sized {
    fn decode<T: Buf>(command: EncapsulationCommand, buffer: &mut T) -> Result<Self, BinaryError>;
}

impl EncapsulationPayloadFromBytes for EncapsulationPayload {
    fn decode<T: Buf>(command: EncapsulationCommand, buffer: &mut T) -> Result<Self, BinaryError> {
        match command {
            EncapsulationCommand::Nop => Ok(EncapsulationPayload::None),
            EncapsulationCommand::ListServices => Ok(EncapsulationPayload::None),
            EncapsulationCommand::ListIdentity => Ok(EncapsulationPayload::None),
            EncapsulationCommand::ListInterfaces => Ok(EncapsulationPayload::None),
            EncapsulationCommand::RegisterSession => Ok(EncapsulationPayload::RegisterSession(
                RegisterSessionData::decode(buffer)?,
            )),
            EncapsulationCommand::UnregisterSession => Ok(EncapsulationPayload::None),
            EncapsulationCommand::SendRRData => Ok(EncapsulationPayload::Cpf(Cpf::decode(buffer)?)),
            EncapsulationCommand::SendUnitData => {
                Ok(EncapsulationPayload::Cpf(Cpf::decode(buffer)?))
            }
            EncapsulationCommand::Unknown(_) => Ok(EncapsulationPayload::None),
        }
    }
}

impl ToBytes for EncapsulationPayload {
    fn encode<T: BufMut>(&self, buffer: &mut T) -> Result<(), BinaryError> {
        match self {
            EncapsulationPayload::None => Ok(()),
            EncapsulationPayload::RegisterSession(data) => data.encode(buffer),
            EncapsulationPayload::Cpf(cpf) => cpf.encode(buffer),
        }
    }

    fn encoded_len(&self) -> usize {
        match self {
            EncapsulationPayload::None => 0,
            EncapsulationPayload::RegisterSession(data) => data.encoded_len(),
            EncapsulationPayload::Cpf(cpf) => cpf.encoded_len(),
        }
    }
}
