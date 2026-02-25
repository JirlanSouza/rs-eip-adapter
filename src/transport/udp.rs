use std::{io, net::Ipv4Addr, sync::Arc};

use futures_util::{sink::SinkExt, stream::StreamExt};
use tokio::{net::UdpSocket, sync::broadcast::Sender};
use tokio_util::udp::UdpFramed;

use super::udp_codec::EncapsulationUdpCodec;
use crate::encap::{
    CastMode, ConnectionContext, EncapsulationHandler, TransportType, handler::HandlerAction,
};

pub struct UdpTransport {
    ip_address: Ipv4Addr,
    port: u16,
    handler: Arc<EncapsulationHandler>,
    shutdown_tx: Arc<Sender<()>>,
}

impl UdpTransport {
    pub async fn new(
        handler: Arc<EncapsulationHandler>,
        port: u16,
        shutdown_tx: Arc<Sender<()>>,
    ) -> io::Result<Self> {
        Ok(Self {
            ip_address: Ipv4Addr::UNSPECIFIED,
            port,
            handler,
            shutdown_tx,
        })
    }

    pub async fn listen(&self) -> io::Result<()> {
        let socket = match UdpSocket::bind((self.ip_address, self.port)).await {
            Ok(socket) => {
                log::info!("UDP socket bound to {}", socket.local_addr()?);
                socket
            }
            Err(err) => return Err(err),
        };

        log::info!(
            "Listening for UDP broadcast packets on {}",
            socket.local_addr()?
        );

        let mut framed = UdpFramed::new(socket, EncapsulationUdpCodec::new());
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        loop {
            tokio::select! {
                _ = self.handle_framed(&mut framed) => {},
                _ = shutdown_rx.recv() => {
                    log::info!("UDP transport shutting down");
                    break;
                }
            }
        }
        Ok(())
    }

    async fn handle_framed(&self, framed: &mut UdpFramed<EncapsulationUdpCodec>) {
        let frame_result_opt = framed.next().await;

        if frame_result_opt.is_none() {
            log::error!("Failed to receive UDP frame");
            return;
        }

        let frame_result = frame_result_opt.unwrap();
        if let Ok((mut frame, peer_addr)) = frame_result {
            let mut context =
                ConnectionContext::new(peer_addr, TransportType::UDP(CastMode::Broadcast));

            match self.handler.handle(&mut frame, &mut context) {
                Ok(HandlerAction::Reply(reply)) => {
                    if let Err(err) = framed.send((reply, peer_addr)).await {
                        log::error!("Failed to send reply to {} : {}", peer_addr, err);
                    }
                }
                Ok(HandlerAction::None) => {
                    log::info!("No reply to send to: {}", peer_addr);
                }
                Ok(HandlerAction::DropConnection) => {
                    log::warn!("Received a HandlerAction::DropConnection in UDP transport");
                }
                Err(err) => {
                    log::error!("Failed to handle request from {} : {}", peer_addr, err);
                }
            }
            return;
        }

        log::error!(
            "Failed to decode UDP datagram: {}",
            frame_result.unwrap_err()
        );
    }
}
