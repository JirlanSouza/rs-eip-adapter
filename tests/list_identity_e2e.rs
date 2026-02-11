use crate::common::{
    eip_stack::{self, DEFAULT_IDENTITY_INFO},
    udp,
};
use bytes::{BufMut, BytesMut};
use rs_eip_adapter::encap::{
    command::{self, EncapsulationCommand},
    header::{ENCAPSULATION_HEADER_SIZE, EncapsulationHeader},
};

mod common;

const DEFAULT_REQUEST_HEADER: EncapsulationHeader = EncapsulationHeader {
    command: command::EncapsulationCommand::Nop,
    length: 0x00,
    session_handle: 0x00000000,
    status: 0x00000000,
    context: [0u8; 8],
    options: 0x00000000,
};

#[tokio::test]
async fn list_identity_success_e2e() {
    let context = eip_stack::run_stack(eip_stack::DEFAULT_IDENTITY_INFO)
        .await
        .expect("Error on run Eip stack");

    let request_header = EncapsulationHeader {
        command: command::EncapsulationCommand::ListIdentity,
        ..DEFAULT_REQUEST_HEADER
    };

    let mut request_buf = BytesMut::with_capacity(ENCAPSULATION_HEADER_SIZE);
    let _ = request_header
        .encode(&mut request_buf)
        .expect("Error on encode request header");

    let response = udp::send_and_receive(
        &format!("127.0.0.1:{}", context.udp_broadcast_port),
        request_buf.freeze(),
        2000,
    )
    .await;

    assert!(response.is_some());

    let mut response_buf = response.unwrap();
    let response_header =
        EncapsulationHeader::decode(&mut response_buf).expect("Error on decode response header");

    assert_eq!(
        response_header,
        EncapsulationHeader {
            length: 61,
            ..request_header
        }
    );

    let cpf_item_count = u16::from_le_bytes([response_buf[0], response_buf[1]]);
    assert_eq!(cpf_item_count, 1);

    let cpf_item_id = u16::from_le_bytes([response_buf[2], response_buf[3]]);
    let list_identity_cpf_item_id = 0x000C;
    assert_eq!(cpf_item_id, list_identity_cpf_item_id);

    let cpf_item_length = u16::from_le_bytes([response_buf[4], response_buf[5]]);
    assert_eq!(cpf_item_length, 55);

    let encapsulation_protocol_version = u16::from_le_bytes([response_buf[6], response_buf[7]]);
    assert_eq!(encapsulation_protocol_version, 1);

    let sin_family = u16::from_be_bytes([response_buf[8], response_buf[9]]);
    assert_eq!(sin_family, 2);

    let sin_port = u16::from_be_bytes([response_buf[10], response_buf[11]]);
    assert_eq!(sin_port, 0xAF12);

    let sin_addr = u32::from_be_bytes([
        response_buf[12],
        response_buf[13],
        response_buf[14],
        response_buf[15],
    ]);
    assert_eq!(sin_addr, eip_stack::LOCAL_ADDRESS.to_bits());

    let sin_zero = &response_buf[16..24];
    assert_eq!(sin_zero, &[0; 8]);

    let vendor_id = u16::from_le_bytes([response_buf[24], response_buf[25]]);
    assert_eq!(vendor_id, DEFAULT_IDENTITY_INFO.vendor_id);

    let device_type = u16::from_le_bytes([response_buf[26], response_buf[27]]);
    assert_eq!(device_type, DEFAULT_IDENTITY_INFO.device_type);

    let product_code = u16::from_le_bytes([response_buf[28], response_buf[29]]);
    assert_eq!(product_code, DEFAULT_IDENTITY_INFO.product_code);

    let revision_major = response_buf[30];
    assert_eq!(revision_major, DEFAULT_IDENTITY_INFO.revision_major);

    let revision_minor = response_buf[31];
    assert_eq!(revision_minor, DEFAULT_IDENTITY_INFO.revision_minor);

    let status = response_buf[32];
    assert_eq!(status, 0x00);

    let serial_number = u32::from_le_bytes([
        response_buf[34],
        response_buf[35],
        response_buf[36],
        response_buf[37],
    ]);
    assert_eq!(serial_number, DEFAULT_IDENTITY_INFO.serial_number);

    let name_length = response_buf[38];
    assert_eq!(name_length, DEFAULT_IDENTITY_INFO.product_name.len() as u8);
    assert_eq!(
        &response_buf[39..60],
        DEFAULT_IDENTITY_INFO.product_name.as_bytes()
    );

    let state = response_buf[60];
    assert_eq!(state, 0x00);
    context.stop().await;
}

#[tokio::test]
async fn list_identity_with_payload_error() {
    let context = eip_stack::run_stack(eip_stack::DEFAULT_IDENTITY_INFO)
        .await
        .expect("Failed to run EIP stack");

    let request_header = EncapsulationHeader {
        command: EncapsulationCommand::ListIdentity,
        length: 4,
        ..DEFAULT_REQUEST_HEADER
    };

    let mut request_buf = BytesMut::with_capacity(ENCAPSULATION_HEADER_SIZE + 4);
    request_header
        .encode(&mut request_buf)
        .expect("Failed to encode request header");
    request_buf.put_u32_le(0x12345678);

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
