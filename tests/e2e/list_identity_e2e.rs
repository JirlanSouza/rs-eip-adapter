use bytes::{BufMut, BytesMut};

use crate::common::{
    eip_stack::{self, DEFAULT_IDENTITY_INFO},
    udp,
};
use rs_eip_adapter::{
    common::binary::{FromBytes, ToBytes},
    encap::{
        command::{self, EncapsulationCommand},
        header::{EncapsulationHeader, EncapsulationStatus},
    },
};

const CPF_HEADER_LEN: u16 = 6;
const IDENTITY_ITEM_FIXED_DATA_LEN: u16 = 34;
const IDENTITY_ITEM_PRODUCT_NAME_LEN: u16 = DEFAULT_IDENTITY_INFO.product_name.len() as u16 + 1;
const REPLY_DEFAULT_IDENTITY_LENGTH: u16 =
    CPF_HEADER_LEN + IDENTITY_ITEM_FIXED_DATA_LEN + IDENTITY_ITEM_PRODUCT_NAME_LEN;

const DEFAULT_REQUEST_HEADER: EncapsulationHeader = EncapsulationHeader {
    command: command::EncapsulationCommand::Nop,
    length: 0x00,
    session_handle: 0x00000000,
    status: EncapsulationStatus::Success,
    context: [0u8; 8],
    options: 0x00000000,
};

#[tokio::test]
async fn list_identity_success_reply_is_correct() {
    let context = eip_stack::run_stack(eip_stack::DEFAULT_IDENTITY_INFO)
        .await
        .expect("Error on run Eip stack");

    let request_header = EncapsulationHeader {
        command: command::EncapsulationCommand::ListIdentity,
        ..DEFAULT_REQUEST_HEADER
    };

    let mut request_buf = BytesMut::with_capacity(EncapsulationHeader::LEN);
    let _ = request_header
        .encode(&mut request_buf)
        .expect("Error on encode request header");

    let reply = udp::send_and_receive(
        &format!("127.0.0.1:{}", context.udp_broadcast_port),
        request_buf.freeze(),
        eip_stack::TEST_TIMEOUT_MS,
    )
    .await;

    assert!(reply.is_some());

    let mut reply_buf = reply.unwrap();
    let reply_header =
        EncapsulationHeader::decode(&mut reply_buf).expect("Error on decode reply header");

    assert_eq!(
        reply_header,
        EncapsulationHeader {
            length: REPLY_DEFAULT_IDENTITY_LENGTH,
            ..request_header
        }
    );

    let cpf_item_count = u16::from_le_bytes([reply_buf[0], reply_buf[1]]);
    assert_eq!(cpf_item_count, 1);

    let cpf_item_id = u16::from_le_bytes([reply_buf[2], reply_buf[3]]);
    let list_identity_cpf_item_id = 0x000C;
    assert_eq!(cpf_item_id, list_identity_cpf_item_id);

    let cpf_item_length = u16::from_le_bytes([reply_buf[4], reply_buf[5]]);
    assert_eq!(
        cpf_item_length,
        IDENTITY_ITEM_FIXED_DATA_LEN + IDENTITY_ITEM_PRODUCT_NAME_LEN
    );

    let encapsulation_protocol_version = u16::from_le_bytes([reply_buf[6], reply_buf[7]]);
    assert_eq!(encapsulation_protocol_version, 1);

    let sin_family = u16::from_be_bytes([reply_buf[8], reply_buf[9]]);
    assert_eq!(sin_family, 2);

    let sin_port = u16::from_be_bytes([reply_buf[10], reply_buf[11]]);
    assert_eq!(sin_port, 0xAF12);

    let sin_addr = u32::from_be_bytes([reply_buf[12], reply_buf[13], reply_buf[14], reply_buf[15]]);
    assert_eq!(sin_addr, eip_stack::LOCAL_ADDRESS.to_bits());

    let sin_zero = &reply_buf[16..24];
    assert_eq!(sin_zero, &[0; 8]);

    let vendor_id = u16::from_le_bytes([reply_buf[24], reply_buf[25]]);
    assert_eq!(vendor_id, DEFAULT_IDENTITY_INFO.vendor_id);

    let device_type = u16::from_le_bytes([reply_buf[26], reply_buf[27]]);
    assert_eq!(device_type, DEFAULT_IDENTITY_INFO.device_type);

    let product_code = u16::from_le_bytes([reply_buf[28], reply_buf[29]]);
    assert_eq!(product_code, DEFAULT_IDENTITY_INFO.product_code);

    let revision_major = reply_buf[30];
    assert_eq!(revision_major, DEFAULT_IDENTITY_INFO.revision_major);

    let revision_minor = reply_buf[31];
    assert_eq!(revision_minor, DEFAULT_IDENTITY_INFO.revision_minor);

    let status = reply_buf[32];
    assert_eq!(status, 0x00);

    let serial_number =
        u32::from_le_bytes([reply_buf[34], reply_buf[35], reply_buf[36], reply_buf[37]]);
    assert_eq!(serial_number, DEFAULT_IDENTITY_INFO.serial_number);

    let name_length = reply_buf[38];
    assert_eq!(name_length, DEFAULT_IDENTITY_INFO.product_name.len() as u8);
    let name_end_index = 38 + IDENTITY_ITEM_PRODUCT_NAME_LEN as usize;
    assert_eq!(
        &reply_buf[39..name_end_index],
        DEFAULT_IDENTITY_INFO.product_name.as_bytes()
    );

    let state = reply_buf[name_end_index];
    assert_eq!(state, 0x00);
    context.stop().await;
}

#[tokio::test]
async fn list_identity_with_invalid_length_and_payload_reply_status_is_error() {
    let context = eip_stack::run_stack(eip_stack::DEFAULT_IDENTITY_INFO)
        .await
        .expect("Failed to run EIP stack");

    let request_header = EncapsulationHeader {
        command: EncapsulationCommand::ListIdentity,
        length: 4,
        ..DEFAULT_REQUEST_HEADER
    };

    let mut request_buf = BytesMut::with_capacity(EncapsulationHeader::LEN + 4);
    request_header
        .encode(&mut request_buf)
        .expect("Failed to encode request header");
    request_buf.put_u32_le(0x12345678);

    let reply = udp::send_and_receive(
        &format!("127.0.0.1:{}", context.udp_broadcast_port),
        request_buf.freeze(),
        eip_stack::TEST_TIMEOUT_MS,
    )
    .await;

    assert!(reply.is_some());
    let mut reply_buf = reply.unwrap();
    let reply_header =
        EncapsulationHeader::decode(&mut reply_buf).expect("Failed to decode reply header");

    assert_eq!(reply_header.status, EncapsulationStatus::InvalidLength);

    context.stop().await;
}
