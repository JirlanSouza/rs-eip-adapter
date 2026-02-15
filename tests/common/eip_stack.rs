use crate::common::udp;
use rs_eip_adapter::{
    cip::cip_identity::IdentityInfo,
    eip_stack::{EipStack, EipStackBuilder},
};
use std::{io::Error, net::Ipv4Addr, sync::Arc};
use tokio::{task::JoinHandle, time};

pub const DEFAULT_IDENTITY_INFO: IdentityInfo = IdentityInfo {
    vendor_id: 0x0000,
    device_type: 0x0000,
    product_code: 0x0000,
    revision_major: 0x00,
    revision_minor: 0x00,
    serial_number: 0x00000000,
    product_name: "Rust EIP Adapter test",
};
pub const LOCAL_ADDRESS: Ipv4Addr = Ipv4Addr::LOCALHOST;
pub const INVALID_COMMAND_ERROR_CODE: u32 = 0x0001;
pub const INVALID_LENGTH_ERROR_CODE: u32 = 0x0065;
pub const TEST_TIMEOUT_MS: u16 = 2000;
const SERVER_STARTUP_TIMEOUT_MS: u16 = 100;

pub struct TestContext {
    eip_stack: Arc<EipStack>,
    server_handle: JoinHandle<Result<(), Error>>,
    pub udp_broadcast_port: u16,
}

impl TestContext {
    pub async fn stop(self) {
        _ = self.eip_stack.stop();
        _ = self
            .server_handle
            .await
            .expect("Error on server handle join");
    }
}

pub async fn run_stack(identity_info: IdentityInfo) -> Result<TestContext, Error> {
    let _ = env_logger::try_init();
    let local_address = LOCAL_ADDRESS;
    let udp_broadcast_port = udp::get_free_port().await;

    let eip_stack = Arc::new(
        EipStackBuilder::new(identity_info)
            .with_address(local_address)
            .with_udp_broadcast_port(udp_broadcast_port)
            .build()
            .await
            .expect("Error build Eip stack"),
    );
    let eip_stack_clone = eip_stack.clone();
    let server_handle = tokio::spawn(async move { eip_stack_clone.start().await });

    tokio::time::sleep(time::Duration::from_millis(
        SERVER_STARTUP_TIMEOUT_MS as u64,
    ))
    .await;
    Ok(TestContext {
        eip_stack,
        server_handle,
        udp_broadcast_port,
    })
}
