use crate::encap::session_handler::SessionHandler;
use crate::transport::tcp_connection::TcpConnection;
use std::{
    io,
    net::{Ipv4Addr, SocketAddr},
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast::Receiver,
};

pub const MAX_TCP_BUFFER_SIZE: usize = 2048;
pub struct TcpTransport {
    tcp_listener: TcpListener,
    handler: SessionHandler,
    shutdown: Receiver<()>,
}

impl TcpTransport {
    pub async fn new(
        command_dispatcher: SessionHandler,
        port: u16,
        shutdown: Receiver<()>,
    ) -> io::Result<Self> {
        let tcp_listener = match TcpListener::bind((Ipv4Addr::UNSPECIFIED, port)).await {
            Ok(listener) => {
                log::info!("TCP socket bound to {}", listener.local_addr()?);
                listener
            }
            Err(err) => return Err(err),
        };

        Ok(Self {
            tcp_listener,
            handler: command_dispatcher,
            shutdown,
        })
    }

    pub async fn listen_tcp(&mut self) -> io::Result<()> {
        log::info!(
            "Listening for TCP packets on {}",
            self.tcp_listener.local_addr()?
        );
        loop {
            tokio::select! {
                result = self.tcp_listener.accept() => {
                    match result {
                        Ok((stream, src)) => {
                            self.handle_connection(stream, src).await;
                        }
                        Err(err) => {
                            log::error!("Failed to accept TCP connection: {}", err);
                        }
                    }
                }
                _ = self.shutdown.recv() => {
                    log::info!("TCP transport shutting down");
                    break;
                }
            }
        }
        Ok(())
    }

    async fn handle_connection(&mut self, stream: TcpStream, src: SocketAddr) {
        let mut connection = TcpConnection::new(src, stream, MAX_TCP_BUFFER_SIZE);

        loop {
            tokio::select! {
                result = connection.read_message() => {
                    let encapsulation =  match result {
                        Ok(encapsulation) => encapsulation,
                        Err(e) => {
                            log::error!("Failed to read message from stream: {}, {}", src, e);
                            break;
                        }
                    };

                    let handle_result = self
                        .handler
                        .handle(encapsulation);

                    if let Some(reply_buf) = handle_result {
                        log::info!(
                            "Sending reply to {:?} with {} bytes",
                            src.ip(),
                            reply_buf.len()
                        );
                        match connection.write(&reply_buf).await {
                            Ok(_) => {
                                log::info!(
                                    "Reply {} bytes sent successfully to {:?}",
                                    reply_buf.len(),
                                    src
                                );
                            }
                            Err(e) => log::error!("Failed to send reply to {:?}: {}", src.ip(), e),
                        }
                        continue;
                    }

                    log::info!("No bytes to reply to {:?}", src.ip());
                }
                _ = self.shutdown.recv() => {
                    log::info!("TCP connection shutting down: {}", src);
                    let _ = connection.close().await;
                    break;
                }
            }
        }
    }
}
