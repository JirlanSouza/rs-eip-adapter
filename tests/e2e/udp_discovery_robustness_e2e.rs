use bytes::{BufMut, BytesMut};

use crate::common::{eip_stack, udp};
use rs_eip_adapter::encap::header::EncapsulationHeader;

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
