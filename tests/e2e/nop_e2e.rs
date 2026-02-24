use bytes::BytesMut;

use rs_eip_adapter::{
    common::binary::ToBytes,
    encap::{
        command::EncapsulationCommand,
        header::{EncapsulationHeader, EncapsulationStatus},
    },
};

use crate::common::{eip_stack, tcp};

const DEFAULT_REQUEST_HEADER: EncapsulationHeader = EncapsulationHeader {
    command: EncapsulationCommand::Nop,
    length: 0x00,
    session_handle: 0x00000000,
    status: EncapsulationStatus::Success,
    context: [0u8; 8],
    options: 0x00000000,
};

#[tokio::test]
async fn nop_no_data_sends_no_reply() {
    let context = eip_stack::run_stack(eip_stack::DEFAULT_IDENTITY_INFO)
        .await
        .expect("Error on run Eip stack");

    let request_header = EncapsulationHeader {
        ..DEFAULT_REQUEST_HEADER
    };

    let mut request_buf = BytesMut::with_capacity(EncapsulationHeader::LEN);
    let _ = request_header
        .encode(&mut request_buf)
        .expect("Error on encode request header");

    let reply = tcp::send_and_receive(
        &format!("127.0.0.1:{}", context.tcp_port),
        request_buf.freeze(),
        1,
        eip_stack::TEST_TIMEOUT_MS,
    )
    .await;
    let _ = context.stop().await;

    assert!(reply.is_none());
}

#[tokio::test]
async fn nop_with_data_sends_no_reply() {
    let context = eip_stack::run_stack(eip_stack::DEFAULT_IDENTITY_INFO)
        .await
        .expect("Error on run Eip stack");

    let request_header = EncapsulationHeader {
        length: 0x04,
        ..DEFAULT_REQUEST_HEADER
    };

    let mut request_buf = BytesMut::with_capacity(EncapsulationHeader::LEN + 4);
    let _ = request_header
        .encode(&mut request_buf)
        .expect("Error on encode request header");
    request_buf.extend_from_slice(&[0u8; 4]);

    let reply = tcp::send_and_receive(
        &format!("127.0.0.1:{}", context.tcp_port),
        request_buf.freeze(),
        1,
        eip_stack::TEST_TIMEOUT_MS,
    )
    .await;
    let _ = context.stop().await;

    assert!(reply.is_none());
}
