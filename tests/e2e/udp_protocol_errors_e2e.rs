use bytes::{BufMut, BytesMut};

use crate::common::{eip_stack, udp};
use rs_eip_adapter::{
    common::binary::{FromBytes, ToBytes},
    encap::{
        command::EncapsulationCommand,
        header::{EncapsulationHeader, EncapsulationStatus},
    },
};

const DEFAULT_REQUEST_HEADER: EncapsulationHeader = EncapsulationHeader {
    command: EncapsulationCommand::Nop,
    length: 0x00,
    session_handle: 0x00,
    status: EncapsulationStatus::Success,
    context: [0u8; 8],
    options: 0,
};

#[tokio::test]
async fn invalid_command_over_udp_returns_error() {
    let context = eip_stack::run_stack(eip_stack::DEFAULT_IDENTITY_INFO)
        .await
        .expect("Failed to run EIP stack");

    let request_header = EncapsulationHeader {
        command: EncapsulationCommand::SendRRData,
        ..DEFAULT_REQUEST_HEADER
    };

    let mut request_buf = BytesMut::with_capacity(EncapsulationHeader::LEN);
    request_header
        .encode(&mut request_buf)
        .expect("Failed to encode request header");

    let reply = udp::send_and_receive(
        &format!("127.0.0.1:{}", context.udp_broadcast_port),
        request_buf.freeze(),
        eip_stack::TEST_TIMEOUT_MS,
    )
    .await;
    context.stop().await;

    assert!(reply.is_some());
    let mut reply_buf = reply.unwrap();
    let response_header =
        EncapsulationHeader::decode(&mut reply_buf).expect("Failed to decode reply header");

    assert_eq!(response_header.command, request_header.command);
    assert_eq!(
        response_header.status,
        EncapsulationStatus::InvalidOrUnsupportedCommand
    );
    assert_eq!(response_header.length, 0);
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

    let mut request_buf = BytesMut::with_capacity(EncapsulationHeader::LEN);
    request_header
        .encode(&mut request_buf)
        .expect("Failed to encode request header");

    let reply = udp::send_and_receive(
        &format!("127.0.0.1:{}", context.udp_broadcast_port),
        request_buf.freeze(),
        eip_stack::TEST_TIMEOUT_MS,
    )
    .await;

    assert!(reply.is_some());
    let mut reply_buf = reply.unwrap();
    let response_header =
        EncapsulationHeader::decode(&mut reply_buf).expect("Failed to decode reply header");

    assert_eq!(response_header.command, request_header.command);
    assert_eq!(
        response_header.status,
        EncapsulationStatus::InvalidOrUnsupportedCommand
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

    let mut request_buf = BytesMut::with_capacity(EncapsulationHeader::LEN + 5);
    request_header
        .encode(&mut request_buf)
        .expect("Failed to encode request header");
    request_buf.put_slice(&[0u8; 5]);

    let reply = udp::send_and_receive(
        &format!("127.0.0.1:{}", context.udp_broadcast_port),
        request_buf.freeze(),
        eip_stack::TEST_TIMEOUT_MS,
    )
    .await;

    assert!(reply.is_some());
    let mut reply_buf = reply.unwrap();
    let response_header =
        EncapsulationHeader::decode(&mut reply_buf).expect("Failed to decode reply header");

    assert_eq!(response_header.status, EncapsulationStatus::InvalidLength);

    context.stop().await;
}
