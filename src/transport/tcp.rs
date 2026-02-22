use futures_util::{sink::SinkExt, stream::StreamExt};
use std::{
    io,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast::Sender,
};
use tokio_util::codec::Framed;

use crate::{
    encap::handler::{ConnectionContext, EncapsulationHandler, TransportType},
    transport::codec::EncapsulationCodec,
};

pub struct TcpTransport {
    tcp_listener: TcpListener,
    handler: Arc<EncapsulationHandler>,
    shutdown: Arc<Sender<()>>,
}

impl TcpTransport {
    pub async fn new(
        handler: Arc<EncapsulationHandler>,
        port: u16,
        shutdown: Arc<Sender<()>>,
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
            handler,
            shutdown,
        })
    }

    pub async fn listen(&mut self) -> io::Result<()> {
        log::info!(
            "Listening for TCP packets on: {}",
            self.tcp_listener.local_addr()?
        );

        let mut accept_shutdown_rx = self.shutdown.subscribe();
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
                _ = accept_shutdown_rx.recv() => {
                    log::info!("TCP transport shutting down");
                    break;
                }
            }
        }
        Ok(())
    }

    async fn handle_connection(&mut self, stream: TcpStream, src: SocketAddr) {
        let mut context = ConnectionContext::new(TransportType::TCP);
        let mut framed = Framed::new(stream, EncapsulationCodec::new());
        let mut connection_shutdown_rx = self.shutdown.subscribe();

        loop {
            tokio::select! {
                _ = self.handle_framed(&mut framed, &mut context) => {},
                _ = connection_shutdown_rx.recv() => {
                    log::info!("TCP connection shutting down: {}", src);
                    let _ = framed.close().await;
                    break;
                }
            }
        }
    }

    async fn handle_framed(
        &self,
        framed: &mut Framed<TcpStream, EncapsulationCodec>,
        context: &mut ConnectionContext,
    ) {
        let frame_result_opt = framed.next().await;

        if frame_result_opt.is_none() {
            log::error!("Failed to receive TCP frame");
            return;
        }

        let frame_result = frame_result_opt.unwrap();
        if let Ok(mut frame) = frame_result {
            match self.handler.handle(&mut frame, context) {
                Ok(reply) => {
                    if let Err(err) = framed.send(reply).await {
                        log::error!("Failed to send reply: {}", err);
                    }
                }
                Err(err) => {
                    log::error!("Failed to handle request: {}", err);
                }
            }
            return;
        }

        log::error!(
            "Failed to decode TCP datagram: {}",
            frame_result.unwrap_err()
        );
    }
}
