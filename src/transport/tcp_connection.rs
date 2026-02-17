use bytes::BytesMut;
use std::{io, net::SocketAddr};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::encap::{
    Encapsulation, error::FrameError, header::{ENCAPSULATION_HEADER_SIZE, EncapsulationHeader}
};

pub struct TcpConnection {
    socket_address: SocketAddr,
    stream: TcpStream,
    buffer: BytesMut,
}

impl TcpConnection {
    pub fn new(socket_address: SocketAddr, stream: TcpStream, buffer_capacity: usize) -> Self {
        log::debug!("New TCP connection from {}, buffer capacity: {}", socket_address, buffer_capacity);
        assert!(buffer_capacity >= ENCAPSULATION_HEADER_SIZE);
        Self {
            socket_address,
            stream,
            buffer: BytesMut::with_capacity(buffer_capacity),
        }
    }

    pub async fn read_message(&mut self) -> Result<Encapsulation, FrameError> {
        match self.read_header().await {
            Ok(header) => {
                _ = self.read_data(header.length as usize).await.map_err(|_| FrameError::Inconplete(0));
                let payload = self.buffer.split_to(header.length as usize).freeze();
                Encapsulation::new(header, payload)
            }
            Err(e) => Err(e),
        }
    }

    pub async fn read_header(&mut self) -> Result<EncapsulationHeader, FrameError> {
        self.buffer.clear();
        _ = self.read_data(ENCAPSULATION_HEADER_SIZE).await.map_err(|_| FrameError::Inconplete(0));

        let mut header_buffer = self.buffer.split_to(ENCAPSULATION_HEADER_SIZE).freeze();
        EncapsulationHeader::decode(&mut header_buffer)
    }

    pub async fn read_data(&mut self, len: usize) -> io::Result<()> {
        log::debug!("Reading {} bytes from {}", len, self.socket_address);
        if len == 0 {
            return Ok(());
        }

        if len + ENCAPSULATION_HEADER_SIZE > self.buffer.capacity() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Not enough space in buffer",
            ));
        }

        self.buffer.resize(len, 0);
        match self.stream.read_exact(&mut self.buffer[..len]).await {
            Ok(_) => {
                log::debug!("{} bytes read successfully from {}", len, self.socket_address);
                Ok(())
            }
            Err(e) => self.handle_error(e).await,
        }
    }

    pub async fn write(&mut self, data: &[u8]) -> io::Result<()> {
        log::debug!("Sending {} bytes to {}", data.len(), self.socket_address);
        match self.stream.write_all(data).await {
            Ok(_) => {
                log::debug!("{} bytes sent successfully to {}", data.len(), self.socket_address);
                Ok(())
            }
            Err(e) => self.handle_error(e).await,
        }
    }

    pub async fn close(&mut self) -> io::Result<()> {
        log::debug!("Closing connection to {}", self.socket_address);
        self.stream.shutdown().await
    }

    async fn handle_error(&mut self, e: io::Error) -> io::Result<()> {
        match e.kind() {
            io::ErrorKind::UnexpectedEof => {
                log::warn!("Connection closed by peer: {}", self.socket_address);
                self.close().await
            }
            io::ErrorKind::ConnectionReset => {
                log::warn!("Connection reset by peer: {}", self.socket_address);
                self.close().await
            }
            io::ErrorKind::TimedOut => {
                log::warn!("Connection timeout: {}", self.socket_address);
                self.close().await
            }
            _ => {
                log::error!("Error on TCP stream: {}, {}", self.socket_address, e);
                self.close().await
            }
        }
    }
}
