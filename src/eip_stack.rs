use std::{io, net::Ipv4Addr, sync::Arc};
use tokio::sync::{Mutex, broadcast::Sender};

use crate::{
    cip::{
        cip_class::CipClass,
        cip_identity::{IdentityClass, IdentityInfo},
        registry::Registry,
        tcp_ip_interface::{EIP_RESERVED_PORT, TcpIpInterfaceClass, TcpIpInterfaceInstance},
    },
    encap::{handler::EncapsulationHandler, session_manager::SessionManager},
    transport::{tcp::TcpTransport, udp::UdpTransport},
};

pub struct EipStack {
    registry: Arc<Registry>,
    udp_transport: Arc<Mutex<UdpTransport>>,
    tcp_transport: Arc<Mutex<TcpTransport>>,
    shutdown_tx: Arc<Sender<()>>,
}

impl EipStack {
    pub async fn start(&self) -> io::Result<()> {
        log::info!("Starting EIP stack");

        let udp_transport = self.udp_transport.clone();
        let udp_handle = tokio::spawn(async move {
            _ = udp_transport.lock().await.listen().await;
        });

        let tcp_transport = self.tcp_transport.clone();
        let tcp_handle = tokio::spawn(async move {
            _ = tcp_transport.lock().await.listen().await;
        });

        let shutdown_tx = self.shutdown_tx.clone();
        let shutdown_handle = tokio::spawn(async move {
            _ = EipStack::handle_graceful_shutdown(shutdown_tx).await;
        });

        tokio::try_join!(udp_handle, tcp_handle, shutdown_handle)?;
        Ok(())
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

    async fn handle_graceful_shutdown(shutdown_tx: Arc<Sender<()>>) -> io::Result<()> {
        let mut shutdown_rx = shutdown_tx.subscribe();
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                log::info!("Ctrl+C received, stopping EIP stack");
            }
            _ = shutdown_rx.recv() => {
                log::info!("EIP stack shutting down");
                return Ok(());
            }
        };

        log::info!("Ctrl+C received, stopping EIP stack");

        shutdown_tx.send(()).map_err(|err| {
            log::error!("Error on send shutdown signal: {}", err);
            io::Error::new(io::ErrorKind::Other, "Error on send shutdown signal")
        })?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct EipConfig {
    pub identity: IdentityInfo,
    pub local_address: Ipv4Addr,
    pub tcp_port: u16,
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
                tcp_port: EIP_RESERVED_PORT,
                udp_broadcast_port: EIP_RESERVED_PORT,
            },
            registry: Registry::new(),
        }
    }

    pub fn with_address(mut self, addr: Ipv4Addr) -> Self {
        self.config.local_address = addr;
        self
    }

    pub fn with_tcp_port(mut self, port: u16) -> Self {
        self.config.tcp_port = port;
        self
    }

    pub fn with_udp_broadcast_port(mut self, port: u16) -> Self {
        self.config.udp_broadcast_port = port;
        self
    }

    pub async fn build(mut self) -> io::Result<EipStack> {
        log::info!("Building EIP Stack");
        log::debug!("Building EIP Stack with configuration: {:?}", self.config);
        let identity_class = IdentityClass::new(&self.config.identity);
        self.registry.register(identity_class);
        log::info!("Registering Identity Class");

        let tcp_ip_if_class = Arc::new(TcpIpInterfaceClass::new());
        let tcp_ip_if_instance = Arc::new(TcpIpInterfaceInstance::new(
            Arc::downgrade(&(tcp_ip_if_class.clone() as Arc<dyn CipClass>)),
            self.config.local_address,
        ));
        log::info!("Registering TCP/IP Interface instance");

        tcp_ip_if_class
            .add_instance(tcp_ip_if_instance)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "CIP Registration Error"))?;

        self.registry.register(tcp_ip_if_class);

        let registry = Arc::new(self.registry);
        let shutdown_tx = Arc::new(Sender::new(1));
        let handler = Arc::new(EncapsulationHandler::new(
            registry.clone(),
            Arc::new(SessionManager::new()),
        ));

        let udp_transport = UdpTransport::new(
            handler.clone(),
            self.config.udp_broadcast_port,
            shutdown_tx.clone(),
        )
        .await?;

        let tcp_transport =
            TcpTransport::new(handler, self.config.tcp_port, shutdown_tx.clone()).await?;

        Ok(EipStack {
            registry,
            udp_transport: Arc::new(Mutex::new(udp_transport)),
            tcp_transport: Arc::new(Mutex::new(tcp_transport)),
            shutdown_tx,
        })
    }
}
