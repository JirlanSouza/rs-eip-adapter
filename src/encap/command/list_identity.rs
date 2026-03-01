use std::sync::Arc;

use crate::cip::{
    ClassCode, cip_identity::IdentityInstance, registry::Registry,
    tcp_ip_interface::TcpIpInterfaceInstance,
};
use crate::common::binary::ToBytes;
use crate::encap::Encapsulation;
use crate::encap::error::HandlerError;
use crate::encap::handler::HandlerAction;
use crate::encap::header::EncapsulationStatus;
use crate::encap::{
    cpf::{Cpf, cpf_item::CpfItem, identity_item::IdentityItem},
    error::InternalError,
    header::EncapsulationHeader,
    payload::EncapsulationPayload,
};

pub struct ListIdentityHandler {
    registry: Arc<Registry>,
}

impl ListIdentityHandler {
    pub fn new(registry: Arc<Registry>) -> Self {
        Self { registry }
    }

    pub fn handle(&self, req_header: &EncapsulationHeader) -> Result<HandlerAction, HandlerError> {
        log::info!("Handle ListIdentity request: {:?}", req_header);
        let identity = self
            .registry
            .get_instance::<IdentityInstance>(ClassCode::Identity, 1)
            .map_err(HandlerError::from)?;
        let tcp_ip_if = self
            .registry
            .get_instance::<TcpIpInterfaceInstance>(ClassCode::TcpIpInterface, 1)
            .map_err(HandlerError::from)?;
        log::debug!("List identiry with TCP/IP Interface: {:?}", tcp_ip_if);
        log::debug!("List identity with Identity: {:?}", identity);

        let identity_item = IdentityItem::new(Encapsulation::VERSION, &tcp_ip_if, &identity);
        let mut cpf = Cpf::new();
        cpf.add_item(CpfItem::IdentityItem(identity_item));

        let reply_payload = EncapsulationPayload::Cpf(cpf);
        let reply_header = EncapsulationHeader {
            status: EncapsulationStatus::Success,
            length: reply_payload.encoded_len() as u16,
            ..req_header.clone()
        };
        match Encapsulation::new(reply_header, reply_payload) {
            Ok(encapsulation) => Ok(HandlerAction::Reply(encapsulation)),
            Err(error) => Err(HandlerError::Internal(InternalError::Other(
                error.to_string(),
            ))),
        }
    }
}
