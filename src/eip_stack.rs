use crate::cip::{
    cip_class::CipClass,
    cip_identity::{IdentityClass, IdentityInfo},
    registry::Registry,
    tcp_ip_interface::{TcpIpInterfaceClass, TcpIpInterfaceInstance},
};
use crate::encap::EncapsulationHandler;
use crate::transport::udp::UdpTransport;
use std::sync::Arc;
use std::{io, net::Ipv4Addr};

pub struct EipStack {
    registry: Arc<Registry>,
    udp_transport: UdpTransport,
}

impl EipStack {
    pub async fn start(&self) -> io::Result<()> {
        self.udp_transport.listen_broadcast().await
    }

    pub fn get_registry(&self) -> Arc<Registry> {
        self.registry.clone()
    }
}

pub struct EipConfig {
    pub identity: IdentityInfo,
    pub local_address: Ipv4Addr,
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
            },
            registry: Registry::new(),
        }
    }

    pub fn with_address(mut self, addr: Ipv4Addr) -> Self {
        self.config.local_address = addr;
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
        let handler = EncapsulationHandler::new(registry.clone());
        let udp_transport = UdpTransport::new(handler).await?;

        Ok(EipStack {
            registry,
            udp_transport,
        })
    }
}
