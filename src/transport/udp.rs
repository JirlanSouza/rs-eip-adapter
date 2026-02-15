use crate::encap::{
    Encapsulation,
    broadcast_handler::BroadcastHandler,
    error::{EncapsulationError, FrameError},
    header::{ENCAPSULATION_HEADER_SIZE, EncapsulationHeader},
};
use bytes::BytesMut;
use std::{
    io,
    net::{Ipv4Addr, SocketAddr},
};
use tokio::{net::UdpSocket, sync::broadcast::Receiver};

const MAX_UDP_DATAGRAM_SIZE: usize = 2048;

pub struct UdpTransport {
    broadcast_socket: UdpSocket,
    handler: BroadcastHandler,
}

impl UdpTransport {
    pub async fn new(command_dispatcher: BroadcastHandler, port: u16) -> io::Result<Self> {
        let broadcast_socket = match UdpSocket::bind((Ipv4Addr::UNSPECIFIED, port)).await {
            Ok(socket) => {
                log::info!("UDP socket bound to {}", socket.local_addr()?);
                socket
            }
            Err(err) => return Err(err),
        };

        Ok(Self {
            broadcast_socket,
            handler: command_dispatcher,
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
            log::warn!("Received packet with insufficient length from: {}", src);
            return;
        }

        log::info!(
            "Received packet with: {} bytes from: {}",
            data_buf.len(),
            src
        );

        let mut encapsulation = match Encapsulation::decode(data_buf.freeze()) {
            Ok(encapsulation) => encapsulation,
            Err(FrameError::InvalidLength(header, payload_size)) => {
                log::warn!(
                    "Invalid length in encapsulation: Header={:?}, PayloadSize={}",
                    header,
                    payload_size
                );
                let reply_buf_opt = EncapsulationHeader::create_error_response(
                    header,
                    EncapsulationError::InvalidLength,
                );
                if let Some(reply_buf) = reply_buf_opt {
                    _ = self.send_reply(&reply_buf, src).await;
                    return;
                }
                log::error!(
                    "Failed to create error response for InvalidLength to: {}",
                    src
                );
                return;
            }
            Err(err) => {
                log::error!("Failed to decode encapsulation: {}", err);
                return;
            }
        };

        match self.handler.handle(&mut encapsulation) {
            Some(reply_buf) => {
                _ = self.send_reply(&reply_buf, src).await;
            }
            None => {
                log::info!("No bytes to reply to {}", src);
            }
        }
    }

    async fn send_reply(&self, reply_buf: &[u8], src: SocketAddr) -> io::Result<usize> {
        log::info!("Sending reply to {} with {} bytes", src, reply_buf.len());

        self.broadcast_socket
            .send_to(&reply_buf, src)
            .await
            .inspect(|bytes_sent| {
                log::info!("Reply {} bytes sent successfully to {}", bytes_sent, src);
            })
            .inspect_err(|e| {
                log::error!("Failed to send reply to {}: {}", src, e);
            })
    }
}
