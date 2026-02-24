use bytes::{Buf, BufMut, Bytes};

use crate::common::binary::{BinaryError, FromBytes, ToBytes};
use crate::encap::{
    command::{EncapsulationCommand, RegisterSessionData},
    cpf::Cpf,
};

#[derive(Debug)]
pub enum EncapsulationPayload {
    None,
    Nop(Bytes),
    RegisterSession(RegisterSessionData),
    Cpf(Cpf),
}

pub trait EncapsulationPayloadFromBytes: Sized {
    fn decode<T: Buf>(command: EncapsulationCommand, buffer: &mut T) -> Result<Self, BinaryError>;
}

impl EncapsulationPayloadFromBytes for EncapsulationPayload {
    fn decode<T: Buf>(command: EncapsulationCommand, buffer: &mut T) -> Result<Self, BinaryError> {
        match command {
            EncapsulationCommand::Nop => Ok(EncapsulationPayload::Nop(
                buffer.copy_to_bytes(buffer.remaining()),
            )),
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
            EncapsulationCommand::IndicateStatus => Ok(EncapsulationPayload::None),
            EncapsulationCommand::Cancel => Ok(EncapsulationPayload::None),
            EncapsulationCommand::Unknown(_) => Ok(EncapsulationPayload::None),
        }
    }
}

impl ToBytes for EncapsulationPayload {
    fn encode<T: BufMut>(&self, buffer: &mut T) -> Result<(), BinaryError> {
        match self {
            EncapsulationPayload::None => Ok(()),
            EncapsulationPayload::Nop(data) => Ok(buffer.put(data.as_ref())),
            EncapsulationPayload::RegisterSession(data) => data.encode(buffer),
            EncapsulationPayload::Cpf(cpf) => cpf.encode(buffer),
        }
    }

    fn encoded_len(&self) -> usize {
        match self {
            EncapsulationPayload::None => 0,
            EncapsulationPayload::Nop(data) => data.len(),
            EncapsulationPayload::RegisterSession(data) => data.encoded_len(),
            EncapsulationPayload::Cpf(cpf) => cpf.encoded_len(),
        }
    }
}
