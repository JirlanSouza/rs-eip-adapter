use env_logger;
use rs_eip_adapter::{cip::cip_identity::IdentityInfo, eip_stack::EipStackBuilder};
use std::{io, net::Ipv4Addr};

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    env_logger::init();
    log::info!("Starting Rust EIP Adapter");

    let identity_info = IdentityInfo {
        vendor_id: 0x0000,
        device_type: 0x0000,
        product_code: 0x0000,
        revision_major: 0x00,
        revision_minor: 0x00,
        serial_number: 0x00000000,
        product_name: "Rust EIP Adapter".to_string(),
    };

    let eip_stack = EipStackBuilder::new(identity_info)
        .with_address(Ipv4Addr::LOCALHOST)
        .build()
        .await
        .inspect_err(|e| {
            log::error!("Failed to build EIP stack: {}", e);
        })?;

    eip_stack.start().await.inspect_err(|err| {
        log::error!("Error on start Eip stack runtime: {}", err);
    })?;

    Ok(())
}
