use crate::common::{eip_stack, udp};
use bytes::{BufMut, BytesMut};
use rs_eip_adapter::encap::{
    command::EncapsulationCommand,
    header::{ENCAPSULATION_HEADER_SIZE, EncapsulationHeader},
};

mod common;

#[tokio::test]
async fn packet_too_short_sends_no_response() {
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
async fn malformed_header_garbage_sends_error_response() {
    let context = eip_stack::run_stack(eip_stack::DEFAULT_IDENTITY_INFO)
        .await
        .expect("Failed to run EIP stack");

    let mut malformed_header_data = BytesMut::with_capacity(ENCAPSULATION_HEADER_SIZE);
    malformed_header_data.put_slice(&[0xFF; ENCAPSULATION_HEADER_SIZE]);

    let response = udp::send_and_receive(
        &format!("127.0.0.1:{}", context.udp_broadcast_port),
        malformed_header_data.freeze(),
        eip_stack::TEST_TIMEOUT_MS,
    )
    .await;

    assert!(response.is_some());
    let mut response_buffer = response.unwrap();
    let response_header = EncapsulationHeader::decode(&mut response_buffer)
        .expect("Failed to decode response header");

    assert_eq!(
        response_header.command,
        EncapsulationCommand::Unknown(0xFFFF)
    );
    assert_eq!(response_header.status, eip_stack::INVALID_LENGTH_ERROR_CODE);
    assert_eq!(response_header.length, 0);

    context.stop().await;
}
