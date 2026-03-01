use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use bytes::{Bytes, BytesMut};

use rs_eip_adapter::{
    cip::{
        cip_identity::{IdentityClass, IdentityInfo},
        common::object::CipClass,
        registry::Registry,
        tcp_ip_interface::{TcpIpInterfaceClass, TcpIpInterfaceInstance},
    },
    common::binary::ToBytes,
    encap::{
        CastMode, ConnectionContext, EncapsulationHandler, RawEncapsulation, TransportType,
        command::{EncapsulationCommand, register_session::RegisterSessionData},
        handler::HandlerAction,
        header::{EncapsulationHeader, EncapsulationStatus},
        payload::EncapsulationPayload,
        session_manager::SessionManager,
    },
};

fn build_handler() -> EncapsulationHandler {
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
    registry_instance.register(IdentityClass::with_default_instance(&identity_data));

    let tcp_interface_class = Arc::new(TcpIpInterfaceClass::new());
    let tcp_interface_instance = Arc::new(TcpIpInterfaceInstance::new(1, Ipv4Addr::LOCALHOST));
    tcp_interface_class
        .add_instance(tcp_interface_instance)
        .expect("register tcp instance");
    registry_instance.register(tcp_interface_class);

    let session_manager = Arc::new(SessionManager::new());

    EncapsulationHandler::new(Arc::new(registry_instance), session_manager)
}

#[test]
fn handler_reply_status_success_for_list_identity() {
    env_logger::init();
    let handler = build_handler();

    let request_header = EncapsulationHeader {
        command: EncapsulationCommand::ListIdentity,
        length: 0,
        session_handle: 0,
        status: EncapsulationStatus::Success,
        context: [0u8; 8],
        options: 0,
    };

    let mut encapsulation = RawEncapsulation::new(request_header, Bytes::new());
    let mut context = ConnectionContext::new(
        SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0),
        TransportType::UDP(CastMode::Broadcast),
    );

    let action = handler
        .handle(&mut encapsulation, &mut context)
        .expect("Should handle request");

    let HandlerAction::Reply(reply) = action else {
        panic!("Expected HandlerAction::Reply, but got {:?}", action);
    };

    assert_eq!(reply.header.command, EncapsulationCommand::ListIdentity);
    assert_eq!(reply.header.status, EncapsulationStatus::Success);
    assert!(reply.header.length > 0);
}

#[test]
fn handler_should_reply_status_error_for_unsupported_command() {
    let handler = build_handler();

    let request_header = EncapsulationHeader {
        command: EncapsulationCommand::ListInterfaces,
        length: 0,
        session_handle: 0,
        status: EncapsulationStatus::Success,
        context: [0u8; 8],
        options: 0,
    };

    let mut encapsulation = RawEncapsulation::new(request_header, Bytes::new());
    let mut context = ConnectionContext::new(
        SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0),
        TransportType::TCP,
    );

    let action = handler
        .handle(&mut encapsulation, &mut context)
        .expect("Should handle request");

    let HandlerAction::Reply(reply) = action else {
        panic!("Expected HandlerAction::Reply, but got {:?}", action);
    };

    assert_eq!(
        reply.header.status,
        EncapsulationStatus::InvalidOrUnsupportedCommand
    );
}

#[test]
fn handler_should_reply_status_error_for_partially_supported_commands() {
    let handler = build_handler();

    for command in &[
        EncapsulationCommand::ListInterfaces,
        EncapsulationCommand::ListServices,
    ] {
        let request_header = EncapsulationHeader {
            command: command.clone(),
            length: 0,
            session_handle: 0,
            status: EncapsulationStatus::Success,
            context: [0; 8],
            options: 0,
        };

        let mut encapsulation = RawEncapsulation::new(request_header, Bytes::new());
        let mut context = ConnectionContext::new(
            SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0),
            TransportType::UDP(CastMode::Broadcast),
        );

        let action = handler
            .handle(&mut encapsulation, &mut context)
            .expect("Should handle request");

        let HandlerAction::Reply(reply) = action else {
            panic!("Expected HandlerAction::Reply, but got {:?}", action);
        };

        assert_eq!(
            reply.header.status,
            EncapsulationStatus::InvalidOrUnsupportedCommand
        );
    }
}

#[test]
fn handler_should_not_reply_on_list_identity_error() {
    let empty_registry = Arc::new(Registry::new());
    let handler = EncapsulationHandler::new(empty_registry, Arc::new(SessionManager::new()));

    let request_header = EncapsulationHeader {
        command: EncapsulationCommand::ListIdentity,
        length: 0,
        session_handle: 0,
        status: EncapsulationStatus::Success,
        context: [0; 8],
        options: 0,
    };

    let mut encapsulation = RawEncapsulation::new(request_header, Bytes::new());
    let mut context = ConnectionContext::new(
        SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0),
        TransportType::UDP(CastMode::Broadcast),
    );

    let result = handler.handle(&mut encapsulation, &mut context);

    assert!(matches!(result, Err(_)));
}

#[test]
fn handler_should_reply_error_status_for_list_identity_payload_is_not_empty() {
    let handler = build_handler();

    let request_header = EncapsulationHeader {
        command: EncapsulationCommand::ListIdentity,
        length: 4,
        session_handle: 0,
        status: EncapsulationStatus::Success,
        context: [0; 8],
        options: 0,
    };

    let mut req_payload_bytes = BytesMut::with_capacity(4);
    let _ = EncapsulationPayload::RegisterSession(RegisterSessionData {
        protocol_version: 1,
        options: 0,
    })
    .encode(&mut req_payload_bytes)
    .expect("Should encode request payload");

    let mut encapsulation = RawEncapsulation::new(request_header, req_payload_bytes.freeze());
    let mut context = ConnectionContext::new(
        SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0),
        TransportType::UDP(CastMode::Broadcast),
    );

    let action = handler
        .handle(&mut encapsulation, &mut context)
        .expect("Should handle request");

    let HandlerAction::Reply(reply) = action else {
        panic!("Expected HandlerAction::Reply, but got {:?}", action);
    };

    assert_eq!(reply.header.command, EncapsulationCommand::ListIdentity);
    assert_eq!(reply.header.status, EncapsulationStatus::InvalidLength);
    assert!(reply.header.length == 0);
}
