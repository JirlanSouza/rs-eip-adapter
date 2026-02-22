use bytes::BytesMut;

use crate::common::{eip_stack, tcp};
use rs_eip_adapter::{
    common::binary::{FromBytes, ToBytes},
    encap::{
        command::{self, register_session::RegisterSessionData},
        header::{EncapsulationHeader, EncapsulationStatus},
    },
};

const DEFAULT_REQUEST_HEADER: EncapsulationHeader = EncapsulationHeader {
    command: command::EncapsulationCommand::RegisterSession,
    length: 0x00,
    session_handle: 0x00000000,
    status: EncapsulationStatus::Success,
    context: [0u8; 8],
    options: 0x00000000,
};

#[tokio::test]
async fn register_session_success() {
    let context = eip_stack::run_stack(eip_stack::DEFAULT_IDENTITY_INFO)
        .await
        .expect("Error on run Eip stack");

    let request_header = EncapsulationHeader {
        command: command::EncapsulationCommand::RegisterSession,
        length: 0x04,
        ..DEFAULT_REQUEST_HEADER
    };

    let mut request_buf = BytesMut::with_capacity(EncapsulationHeader::LEN + 4);
    let _ = request_header
        .encode(&mut request_buf)
        .expect("Error on encode request header");

    let register_session_data = RegisterSessionData {
        protocol_version: 1,
        options: 0,
    };
    let _ = register_session_data
        .encode(&mut request_buf)
        .expect("Error on encode register session data");

    let reply = tcp::send_and_receive(
        &format!("127.0.0.1:{}", context.tcp_port),
        request_buf.freeze(),
        28,
        1000,
    )
    .await;
    let _ = context.stop().await;

    assert!(reply.is_some());

    let mut reply_buf = reply.unwrap();
    let reply_header =
        EncapsulationHeader::decode(&mut reply_buf).expect("Error on decode reply header");

    assert_eq!(
        reply_header,
        EncapsulationHeader {
            length: 4,
            session_handle: 1,
            ..request_header
        }
    );

    let reply_data =
        RegisterSessionData::decode(&mut reply_buf).expect("Error on decode register session data");

    assert_eq!(reply_data, register_session_data);
}

#[tokio::test]
async fn register_session_invalid_protocol_version() {
    let context = eip_stack::run_stack(eip_stack::DEFAULT_IDENTITY_INFO)
        .await
        .expect("Error on run Eip stack");

    let request_header = EncapsulationHeader {
        command: command::EncapsulationCommand::RegisterSession,
        length: 0x04,
        ..DEFAULT_REQUEST_HEADER
    };

    let mut request_buf = BytesMut::with_capacity(EncapsulationHeader::LEN + 4);
    let _ = request_header
        .encode(&mut request_buf)
        .expect("Error on encode request header");

    let register_session_data = RegisterSessionData {
        protocol_version: 2,
        options: 0,
    };
    let _ = register_session_data
        .encode(&mut request_buf)
        .expect("Error on encode register session data");

    let reply = tcp::send_and_receive(
        &format!("127.0.0.1:{}", context.tcp_port),
        request_buf.freeze(),
        28,
        1000,
    )
    .await;
    let _ = context.stop().await;

    assert!(reply.is_some());

    let mut reply_buf = reply.unwrap();
    let reply_header =
        EncapsulationHeader::decode(&mut reply_buf).expect("Error on decode reply header");

    assert_eq!(
        reply_header,
        EncapsulationHeader {
            length: 4,
            session_handle: 0,
            status: EncapsulationStatus::UnsupportedProtocol,
            ..request_header
        }
    );

    let reply_data =
        RegisterSessionData::decode(&mut reply_buf).expect("Error on decode register session data");

    assert_eq!(
        reply_data,
        RegisterSessionData {
            protocol_version: 1,
            options: 0
        }
    );
}

#[tokio::test]
async fn register_session_invalid_options() {
    let context = eip_stack::run_stack(eip_stack::DEFAULT_IDENTITY_INFO)
        .await
        .expect("Error on run Eip stack");

    let request_header = EncapsulationHeader {
        command: command::EncapsulationCommand::RegisterSession,
        length: 0x04,
        ..DEFAULT_REQUEST_HEADER
    };

    let mut request_buf = BytesMut::with_capacity(EncapsulationHeader::LEN + 4);
    let _ = request_header
        .encode(&mut request_buf)
        .expect("Error on encode request header");

    let register_session_data = RegisterSessionData {
        protocol_version: 1,
        options: 1,
    };
    let _ = register_session_data
        .encode(&mut request_buf)
        .expect("Error on encode register session data");

    let reply = tcp::send_and_receive(
        &format!("127.0.0.1:{}", context.tcp_port),
        request_buf.freeze(),
        28,
        1000,
    )
    .await;
    let _ = context.stop().await;

    assert!(reply.is_some());

    let mut reply_buf = reply.unwrap();
    let reply_header =
        EncapsulationHeader::decode(&mut reply_buf).expect("Error on decode reply header");

    assert_eq!(
        reply_header,
        EncapsulationHeader {
            length: 4,
            session_handle: 0,
            status: EncapsulationStatus::UnsupportedProtocol,
            ..request_header
        }
    );

    let reply_data =
        RegisterSessionData::decode(&mut reply_buf).expect("Error on decode register session data");

    assert_eq!(
        reply_data,
        RegisterSessionData {
            protocol_version: 1,
            options: 0
        }
    );
}
