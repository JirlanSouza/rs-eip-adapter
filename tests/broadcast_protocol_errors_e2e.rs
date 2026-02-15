use crate::common::{eip_stack, udp};
use bytes::{BufMut, BytesMut};
use rs_eip_adapter::encap::{
    command::EncapsulationCommand,
    header::{ENCAPSULATION_HEADER_SIZE, EncapsulationHeader},
};

mod common;

const DEFAULT_REQUEST_HEADER: EncapsulationHeader = EncapsulationHeader {
    command: EncapsulationCommand::Nop,
    length: 0x00,
    session_handle: 0x00,
    status: 0,
    context: [0u8; 8],
    options: 0,
};

#[tokio::test]
async fn invalid_command_over_udp_returns_error() {
    let context = eip_stack::run_stack(eip_stack::DEFAULT_IDENTITY_INFO)
        .await
        .expect("Failed to run EIP stack");

    let request_header = EncapsulationHeader {
        command: EncapsulationCommand::RegisterSession,
        ..DEFAULT_REQUEST_HEADER
    };

    let mut request_buf = BytesMut::with_capacity(ENCAPSULATION_HEADER_SIZE);
    request_header
        .encode(&mut request_buf)
        .expect("Failed to encode request header");

    let response = udp::send_and_receive(
        &format!("127.0.0.1:{}", context.udp_broadcast_port),
        request_buf.freeze(),
        eip_stack::TEST_TIMEOUT_MS,
    )
    .await;

    assert!(response.is_some());
    let mut response_buf = response.unwrap();
    let response_header =
        EncapsulationHeader::decode(&mut response_buf).expect("Failed to decode response header");

    assert_eq!(response_header.command, request_header.command);
    assert_eq!(
        response_header.status,
        eip_stack::INVALID_COMMAND_ERROR_CODE
    );
    assert_eq!(response_header.length, 0);

    context.stop().await;
}

#[tokio::test]
async fn unimplemented_command_over_udp_returns_error() {
    let context = eip_stack::run_stack(eip_stack::DEFAULT_IDENTITY_INFO)
        .await
        .expect("Failed to run EIP stack");

    let request_header = EncapsulationHeader {
        command: EncapsulationCommand::ListServices,
        ..DEFAULT_REQUEST_HEADER
    };

    let mut request_buf = BytesMut::with_capacity(ENCAPSULATION_HEADER_SIZE);
    request_header
        .encode(&mut request_buf)
        .expect("Failed to encode request header");

    let response = udp::send_and_receive(
        &format!("127.0.0.1:{}", context.udp_broadcast_port),
        request_buf.freeze(),
        eip_stack::TEST_TIMEOUT_MS,
    )
    .await;

    assert!(response.is_some());
    let mut response_buf = response.unwrap();
    let response_header =
        EncapsulationHeader::decode(&mut response_buf).expect("Failed to decode response header");

    assert_eq!(response_header.command, request_header.command);
    assert_eq!(
        response_header.status,
        eip_stack::INVALID_COMMAND_ERROR_CODE
    );

    context.stop().await;
}

#[tokio::test]
async fn mismatch_payload_length_returns_error() {
    let context = eip_stack::run_stack(eip_stack::DEFAULT_IDENTITY_INFO)
        .await
        .expect("Failed to run EIP stack");

    let request_header = EncapsulationHeader {
        command: EncapsulationCommand::ListIdentity,
        length: 10,
        ..DEFAULT_REQUEST_HEADER
    };

    let mut request_buf = BytesMut::with_capacity(ENCAPSULATION_HEADER_SIZE + 5);
    request_header
        .encode(&mut request_buf)
        .expect("Failed to encode request header");
    request_buf.put_slice(&[0u8; 5]);

    let response = udp::send_and_receive(
        &format!("127.0.0.1:{}", context.udp_broadcast_port),
        request_buf.freeze(),
        eip_stack::TEST_TIMEOUT_MS,
    )
    .await;

    assert!(response.is_some());
    let mut response_buf = response.unwrap();
    let response_header =
        EncapsulationHeader::decode(&mut response_buf).expect("Failed to decode response header");

    assert_eq!(response_header.status, eip_stack::INVALID_LENGTH_ERROR_CODE);

    context.stop().await;
}
