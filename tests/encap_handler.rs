use bytes::{Bytes, BytesMut};
use rs_eip_adapter::cip::{
    cip_class::CipClass,
    cip_identity::{IdentityClass, IdentityInfo},
    registry::Registry,
    tcp_ip_interface::{TcpIpInterfaceClass, TcpIpInterfaceInstance},
};
use rs_eip_adapter::encap::{
    command::EncapsulationCommand,
    error::EncapsulationError,
    handler::EncapsulationHandler,
    header::{ENCAPSULATION_HEADER_SIZE, EncapsulationHeader},
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
        product_name: "Integration Test Device".to_string(),
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
fn handle_udp_broadcast_should_respond_to_list_identity() {
    let registry = create_registry_for_test();
    let handler = EncapsulationHandler::new(registry);

    let request_header = EncapsulationHeader {
        command: EncapsulationCommand::ListIdentity,
        length: 0,
        session_handle: 0,
        status: 0,
        context: [0u8; 8],
        options: 0,
    };

    let mut request_buffer = BytesMut::with_capacity(ENCAPSULATION_HEADER_SIZE);
    request_header
        .encode(&mut request_buffer)
        .expect("encode request");

    let mut response_bytes = handler
        .handle_udp_broadcast(request_buffer.freeze())
        .expect("handler should return a response");

    let response_header =
        EncapsulationHeader::decode(&mut response_bytes).expect("decode response");

    assert_eq!(response_header.command, EncapsulationCommand::ListIdentity);
    assert_eq!(response_header.status, 0);
    assert!(response_header.length > 0);
}

#[test]
fn handle_udp_broadcast_should_return_error_for_unsupported_command() {
    let registry = create_registry_for_test();
    let handler = EncapsulationHandler::new(registry);

    let unsupported_header = EncapsulationHeader {
        command: EncapsulationCommand::RegisterSession,
        length: 0,
        session_handle: 0,
        status: 0,
        context: [0u8; 8],
        options: 0,
    };

    let mut request_buffer = BytesMut::with_capacity(ENCAPSULATION_HEADER_SIZE);
    unsupported_header
        .encode(&mut request_buffer)
        .expect("encode request");

    let mut response_bytes = handler
        .handle_udp_broadcast(request_buffer.freeze())
        .expect("handler should return an error response");

    let response_header =
        EncapsulationHeader::decode(&mut response_bytes).expect("decode response");

    assert_eq!(
        response_header.status,
        EncapsulationError::InvalidOrUnsupportedCommand.to_u32()
    );
}

#[test]
fn handle_udp_broadcast_should_ignore_truncated_packet() {
    let registry = create_registry_for_test();
    let handler = EncapsulationHandler::new(registry);

    let too_short_packet = Bytes::from(vec![0u8; ENCAPSULATION_HEADER_SIZE - 1]);
    let result = handler.handle_udp_broadcast(too_short_packet);

    assert!(result.is_none());
}
