use tokio::sync::broadcast::Sender;
use crate::cip::{
    cip_class::CipClass,
    cip_identity::{IdentityClass, IdentityInfo},
    registry::Registry,
    tcp_ip_interface::{EIP_RESERVED_PORT, TcpIpInterfaceClass, TcpIpInterfaceInstance},
};
use crate::encap::handler::EncapsulationHandler;
use crate::transport::udp::UdpTransport;
use std::{io, net::Ipv4Addr, sync::Arc};

pub struct EipStack {
    registry: Arc<Registry>,
    shutdown_tx: Sender<()>,
    udp_transport: UdpTransport,
}

impl EipStack {
    pub async fn start(&self) -> io::Result<()> {
        let shutdown_rc = self.shutdown_tx.subscribe();
        self.udp_transport.listen_broadcast(shutdown_rc).await
    }

    pub fn stop(&self) -> io::Result<()> {
        log::info!("Stopping EIP stack");
        self.shutdown_tx.send(()).map_err(|err| {
            log::error!("Error on send shutdown signal: {}", err);
            io::Error::new(io::ErrorKind::Other, "Error on send shutdown signal")
        })?;
        Ok(())
    }

    pub fn get_registry(&self) -> Arc<Registry> {
        self.registry.clone()
    }
}

pub struct EipConfig {
    pub identity: IdentityInfo,
    pub local_address: Ipv4Addr,
    pub udp_broadcast_port: u16,
}

pub struct EipStackBuilder {
    config: EipConfig,
    registry: Registry,
}

impl EipStackBuilder {
    pub fn new(identity: IdentityInfo) -> Self {
        Self {
            config: EipConfig {
                identity,
                local_address: Ipv4Addr::UNSPECIFIED,
                udp_broadcast_port: EIP_RESERVED_PORT,
            },
            registry: Registry::new(),
        }
    }

    pub fn with_address(mut self, addr: Ipv4Addr) -> Self {
        self.config.local_address = addr;
        self
    }

    pub fn with_udp_broadcast_port(mut self, port: u16) -> Self {
        self.config.udp_broadcast_port = port;
        self
    }

    pub async fn build(mut self) -> io::Result<EipStack> {
        let identity_class = IdentityClass::new(&self.config.identity);
        self.registry.register(identity_class);

        let tcp_ip_if_class = Arc::new(TcpIpInterfaceClass::new());
        let tcp_ip_if_instance = Arc::new(TcpIpInterfaceInstance::new(
            Arc::downgrade(&(tcp_ip_if_class.clone() as Arc<dyn CipClass>)),
            self.config.local_address,
        ));

        tcp_ip_if_class
            .add_instance(tcp_ip_if_instance)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "CIP Registration Error"))?;

        self.registry.register(tcp_ip_if_class);

        let registry = Arc::new(self.registry);
        let shutdown_tx = Sender::new(1);
        let handler = EncapsulationHandler::new(registry.clone());
        let udp_transport = UdpTransport::new(handler, self.config.udp_broadcast_port).await?;

        Ok(EipStack {
            registry,
            shutdown_tx,
            udp_transport,
        })
    }
}
