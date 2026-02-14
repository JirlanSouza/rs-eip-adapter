use crate::encap::{handler::EncapsulationHandler, header::ENCAPSULATION_HEADER_SIZE};
use bytes::BytesMut;
use std::{
    io,
    net::{Ipv4Addr, SocketAddr},
};
use tokio::{net::UdpSocket, sync::broadcast::Receiver};

const MAX_UDP_DATAGRAM_SIZE: usize = 2048;

pub struct UdpTransport {
    broadcast_socket: UdpSocket,
    encapsulation_handler: EncapsulationHandler,
}

impl UdpTransport {
    pub async fn new(command_dispatcher: EncapsulationHandler, port: u16) -> io::Result<Self> {
        let broadcast_socket = match UdpSocket::bind((Ipv4Addr::UNSPECIFIED, port)).await {
            Ok(socket) => {
                log::info!("UDP socket bound to {}", socket.local_addr()?);
                socket
            }
            Err(err) => return Err(err),
        };

        Ok(Self {
            broadcast_socket,
            encapsulation_handler: command_dispatcher,
        })
    }

    pub async fn listen_broadcast(&self, mut shutdown: Receiver<()>) -> io::Result<()> {
        log::info!(
            "Listening for UDP broadcast packets on {}",
            self.broadcast_socket.local_addr()?
        );
        let mut receiv_buf = [0u8; MAX_UDP_DATAGRAM_SIZE];
        loop {
            tokio::select! {
                result = self.broadcast_socket.recv_from(&mut receiv_buf) => {
                    match result {
                        Ok((len, src)) => {
                            self.handle_datagram(BytesMut::from(&receiv_buf[..len]), src).await;
                        }
                        Err(err) => {
                            log::error!("Failed to receive UDP broadcast packet: {}", err);
                        }
                    }

                }
                _ = shutdown.recv() => {
                    log::info!("UDP transport shutting down");
                    break;
                }
            }
        }
        Ok(())
    }

    async fn handle_datagram(&self, data_buf: BytesMut, src: SocketAddr) {
        if data_buf.len() < ENCAPSULATION_HEADER_SIZE {
            log::warn!("Received packet with insufficient length from {}", src);
            return;
        }

        log::info!("Received packet with {} bytes from {}", data_buf.len(), src);

        let mut handle_result = self
            .encapsulation_handler
            .handle_udp_broadcast(data_buf.freeze());

        if handle_result.is_none() {
            log::info!("No bytes to reply to {:?}", src.ip());
            return;
        }

        let reply_buf = handle_result.take().unwrap();
        log::info!(
            "Sending reply to {:?} with {} bytes",
            src.ip(),
            reply_buf.len()
        );

        match self.broadcast_socket.send_to(&reply_buf, src).await {
            Ok(_) => {
                log::info!(
                    "Reply {} bytes sent successfully to {:?}",
                    reply_buf.len(),
                    src
                );
            }
            Err(e) => log::error!("Failed to send reply to {:?}: {}", src.ip(), e),
        }
    }
}
