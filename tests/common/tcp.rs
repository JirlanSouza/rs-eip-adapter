use std::io;

use bytes::Bytes;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    time,
};

pub async fn get_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Do not found a free port");

    let port = listener
        .local_addr()
        .expect("Fail on get local address")
        .port();

    port
}

pub async fn send_and_receive(
    server_address: &str,
    request_buf: Bytes,
    receive_len: usize,
    timeout: u16,
) -> Option<Bytes> {
    let mut client = TcpStream::connect(server_address)
        .await
        .expect("Error on connect to server");

    let mut response = vec![0u8; receive_len];
    match time::timeout(std::time::Duration::from_millis(timeout as u64), async {
        client
            .write_all(&request_buf)
            .await
            .expect("Error on send request");
        client.read_exact(&mut response).await
    })
    .await
    {
        Ok(_) => {
            log::debug!("Received response: {:?}", response);
            Some(Bytes::from(response))
        }
        Err(_) => {
            log::warn!("Timeout on receive response");
            None
        }
    }
}

pub struct TcpConnection {
    client: TcpStream,
}

impl TcpConnection {
    pub async fn new(server_address: &str) -> Self {
        let client = TcpStream::connect(server_address)
            .await
            .expect("Error on connect to server");

        Self { client }
    }

    pub async fn is_connected(&self) -> bool {
        let mut buf = [0u8; 1];
        self.client.peek(&mut buf).await.is_ok()
    }

    pub async fn send_and_receive(&mut self, request_buf: Bytes, receive_len: usize, timeout: u16) -> Option<Bytes> {
        let mut response = vec![0u8; receive_len];
        match time::timeout(std::time::Duration::from_millis(timeout as u64), async {
            self.client
                .write_all(&request_buf)
                .await?;
            self.client.read_exact(&mut response).await?;
            Ok::<(), io::Error>(())
        })
        .await
        {
            Ok(Ok(())) => {
                log::debug!("Received response: {:?}", response);
                Some(Bytes::from(response))
            }
            Ok(Err(e)) => {
                log::warn!("Error on receive response: {}", e);
                None
            }
            Err(_) => {
                log::warn!("Timeout on receive response");
                None
            }
        }
    }
}
