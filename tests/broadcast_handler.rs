use bytes::Bytes;
use rs_eip_adapter::cip::{
    cip_class::CipClass,
    cip_identity::{IdentityClass, IdentityInfo},
    registry::Registry,
    tcp_ip_interface::{TcpIpInterfaceClass, TcpIpInterfaceInstance},
};
use rs_eip_adapter::encap::{
    Encapsulation, broadcast_handler::BroadcastHandler, command::EncapsulationCommand,
    error::EncapsulationError, header::EncapsulationHeader,
};
use std::net::Ipv4Addr;
use std::sync::Arc;

fn create_registry_for_test() -> Arc<Registry> {
    let mut registry_instance = Registry::new();

    let identity_data = IdentityInfo {
        vendor_id: 0x01,
        device_type: 0x02,
        product_code: 0x03,
        revision_major: 1,
        revision_minor: 0,
        serial_number: 1234,
        product_name: "Integration Test Device",
    };
    registry_instance.register(IdentityClass::new(&identity_data));

    let tcp_interface_class = Arc::new(TcpIpInterfaceClass::new());
    let tcp_interface_instance = Arc::new(TcpIpInterfaceInstance::new(
        Arc::downgrade(&(tcp_interface_class.clone() as Arc<dyn CipClass>)),
        Ipv4Addr::LOCALHOST,
    ));
    tcp_interface_class
        .add_instance(tcp_interface_instance)
        .expect("register tcp instance");
    registry_instance.register(tcp_interface_class);

    Arc::new(registry_instance)
}

#[test]
fn broadcast_handler_reply_status_success_for_list_identity() {
    let registry = create_registry_for_test();
    let handler = BroadcastHandler::new(registry);

    let request_header = EncapsulationHeader {
        command: EncapsulationCommand::ListIdentity,
        length: 0,
        session_handle: 0,
        status: 0,
        context: [0u8; 8],
        options: 0,
    };

    let mut encapsulation = Encapsulation::new(request_header, Bytes::from(vec![0u8; 0]))
        .expect("Should create new Encapsulation");
    let mut response_bytes = handler
        .handle(&mut encapsulation)
        .expect("Should handle request");

    let response_header =
        EncapsulationHeader::decode(&mut response_bytes).expect("decode response");

    assert_eq!(response_header.command, EncapsulationCommand::ListIdentity);
    assert_eq!(response_header.status, 0);
    assert!(response_header.length > 0);
}

#[test]
fn broadcast_handler_should_reply_status_error_for_unsupported_command() {
    let registry = create_registry_for_test();
    let handler = BroadcastHandler::new(registry);

    let request_header = EncapsulationHeader {
        command: EncapsulationCommand::RegisterSession,
        length: 0,
        session_handle: 0,
        status: 0,
        context: [0u8; 8],
        options: 0,
    };

    let mut encapsulation = Encapsulation::new(request_header, Bytes::from(vec![0u8; 0]))
        .expect("Should create new Encapsulation");
    let mut response_bytes = handler
        .handle(&mut encapsulation)
        .expect("Should handle request");

    let response_header =
        EncapsulationHeader::decode(&mut response_bytes).expect("decode response");

    assert_eq!(
        response_header.status,
        EncapsulationError::InvalidOrUnsupportedCommand.to_u32()
    );
}

#[test]
fn broadcast_handler_should_reply_status_error_for_partially_supported_commands() {
    let registry = create_registry_for_test();
    let handler = BroadcastHandler::new(registry);

    for command in &[
        EncapsulationCommand::ListInterfaces,
        EncapsulationCommand::ListServices,
    ] {
        let request_header = EncapsulationHeader {
            command: command.clone(),
            length: 0,
            session_handle: 0,
            status: 0,
            context: [0; 8],
            options: 0,
        };

        let mut encapsulation = Encapsulation::new(request_header, Bytes::from(vec![0u8; 0]))
            .expect("Should create new Encapsulation");
        let mut response_bytes = handler
            .handle(&mut encapsulation)
            .expect("Should handle request");

        let response_header =
            EncapsulationHeader::decode(&mut response_bytes).expect("decode response");

        assert_eq!(
            response_header.status,
            EncapsulationError::InvalidOrUnsupportedCommand.to_u32()
        );
    }
}

#[test]
fn broadcast_handler_should_not_reply_on_list_identity_error() {
    let empty_registry = Arc::new(Registry::new());
    let handler = BroadcastHandler::new(empty_registry);

    let request_header = EncapsulationHeader {
        command: EncapsulationCommand::ListIdentity,
        length: 0,
        session_handle: 0,
        status: 0,
        context: [0; 8],
        options: 0,
    };

    let mut encapsulation = Encapsulation::new(request_header, Bytes::from(vec![0u8; 0]))
        .expect("Should create new Encapsulation");
    let response_bytes_opt = handler.handle(&mut encapsulation);

    assert!(response_bytes_opt.is_none());
}

#[test]
fn broadcast_handler_should_reply_error_status_for_list_identity_payload_is_not_empty() {
    let registry = create_registry_for_test();
    let handler = BroadcastHandler::new(registry);

    let request_header = EncapsulationHeader {
        command: EncapsulationCommand::ListIdentity,
        length: 4,
        session_handle: 0,
        status: 0,
        context: [0; 8],
        options: 0,
    };

    let mut encapsulation = Encapsulation::new(request_header, Bytes::from(vec![1, 2, 3, 4]))
        .expect("Should create new Encapsulation");
    let mut response_bytes = handler
        .handle(&mut encapsulation)
        .expect("Should handle request");

    let response_header =
        EncapsulationHeader::decode(&mut response_bytes).expect("decode response");

    assert_eq!(response_header.command, EncapsulationCommand::ListIdentity);
    assert_eq!(
        response_header.status,
        EncapsulationError::InvalidLength.to_u32()
    );
    assert!(response_header.length == 0);
}
