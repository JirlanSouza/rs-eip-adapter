use crate::encap::{handler::EncapsulationHandler, header::ENCAPSULATION_HEADER_SIZE};
use bytes::BytesMut;
use std::{io, net::Ipv4Addr};
use tokio::net::UdpSocket;

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

    pub async fn listen_broadcast(&self) -> io::Result<()> {
        let mut receiv_buf = [0u8; 2048];
        loop {
            let (len, src) = self.broadcast_socket.recv_from(&mut receiv_buf).await?;

            if len < ENCAPSULATION_HEADER_SIZE {
                log::warn!("Received packet with insufficient length");
                continue;
            }

            let data_buf = BytesMut::from(&receiv_buf[..len]);
            let handle_result = self
                .encapsulation_handler
                .handle_udp_broadcast(data_buf.freeze());

            if let Some(reply_buf) = handle_result {
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
                            src.ip()
                        );
                    }
                    Err(e) => log::error!("Failed to send reply to {:?}: {}", src.ip(), e),
                }
                continue;
            }

            log::info!("No bytes to reply to {:?}", src.ip());
        }
    }
}
