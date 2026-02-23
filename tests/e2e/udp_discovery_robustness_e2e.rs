use bytes::{BufMut, BytesMut};

use crate::common::{eip_stack, udp};
use rs_eip_adapter::{
    common::binary::ToBytes,
    encap::{
        command::EncapsulationCommand,
        header::{EncapsulationHeader, EncapsulationStatus},
    },
};

const DEFAULT_REQUEST_HEADER: EncapsulationHeader = EncapsulationHeader {
    command: EncapsulationCommand::ListIdentity,
    length: 0x00,
    session_handle: 0x00000000,
    status: EncapsulationStatus::Success,
    context: [0u8; 8],
    options: 0x00000000,
};

#[tokio::test]
async fn packet_too_short_sends_no_reply() {
    let context = eip_stack::run_stack(eip_stack::DEFAULT_IDENTITY_INFO)
        .await
        .expect("Failed to run EIP stack");

    let short_packet_data = BytesMut::from(&[0x01, 0x02, 0x03, 0x04][..]);

    let response = udp::send_and_receive(
        &format!("127.0.0.1:{}", context.udp_broadcast_port),
        short_packet_data.freeze(),
        eip_stack::TEST_TIMEOUT_MS,
    )
    .await;

    assert!(response.is_none());

    context.stop().await;
}

#[tokio::test]
async fn malformed_header_garbage_sends_no_reply() {
    let context = eip_stack::run_stack(eip_stack::DEFAULT_IDENTITY_INFO)
        .await
        .expect("Failed to run EIP stack");

    let mut malformed_header_data = BytesMut::with_capacity(EncapsulationHeader::LEN);
    malformed_header_data.put_slice(&[0xFF; EncapsulationHeader::LEN]);

    let response = udp::send_and_receive(
        &format!("127.0.0.1:{}", context.udp_broadcast_port),
        malformed_header_data.freeze(),
        eip_stack::TEST_TIMEOUT_MS,
    )
    .await;

    assert!(response.is_none());
    context.stop().await;
}

#[tokio::test]
async fn invalid_command_for_udp_sends_no_reply() {
    let context = eip_stack::run_stack(eip_stack::DEFAULT_IDENTITY_INFO)
        .await
        .expect("Failed to run EIP stack");

    let invalid_commands = [
        EncapsulationCommand::Nop,
        EncapsulationCommand::RegisterSession,
        EncapsulationCommand::UnregisterSession,
        EncapsulationCommand::SendRRData,
        EncapsulationCommand::SendUnitData,
    ];

    for command in invalid_commands {
        let mut request_buf = BytesMut::with_capacity(EncapsulationHeader::LEN);
        let request_header = EncapsulationHeader {
            command,
            ..DEFAULT_REQUEST_HEADER
        };
        request_header
            .encode(&mut request_buf)
            .expect("Failed to encode request header");

        let reply = udp::send_and_receive(
            &format!("127.0.0.1:{}", context.udp_broadcast_port),
            request_buf.freeze(),
            eip_stack::TEST_TIMEOUT_MS,
        )
        .await;

        assert!(reply.is_none());
    }

    context.stop().await;
}

#[tokio::test]
async fn invalid_encapsulation_status_sends_no_reply() {
    let context = eip_stack::run_stack(eip_stack::DEFAULT_IDENTITY_INFO)
        .await
        .expect("Failed to run EIP stack");

    let invalid_statuses = [
        EncapsulationStatus::InvalidOrUnsupportedCommand,
        EncapsulationStatus::InvalidLength,
        EncapsulationStatus::InvalidSessionHandle,
    ];

    for status in invalid_statuses {
        let mut request_buf = BytesMut::with_capacity(EncapsulationHeader::LEN);
        let request_header = EncapsulationHeader {
            status,
            ..DEFAULT_REQUEST_HEADER
        };
        request_header
            .encode(&mut request_buf)
            .expect("Failed to encode request header");

        let reply = udp::send_and_receive(
            &format!("127.0.0.1:{}", context.udp_broadcast_port),
            request_buf.freeze(),
            eip_stack::TEST_TIMEOUT_MS,
        )
        .await;

        assert!(reply.is_none());
    }

    context.stop().await;
}
