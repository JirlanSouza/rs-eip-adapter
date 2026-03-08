use bytes::Bytes;
use tokio::{net::UdpSocket, time};

pub async fn get_free_port() -> u16 {
    let listener = UdpSocket::bind("127.0.0.1:0")
        .await
        .expect("It was not be possible to get a free UDP port");

    listener
        .local_addr()
        .expect("Failure on retrive the local addr")
        .port()
}

pub async fn send_and_receive(
    server_address: &str,
    request_buf: Bytes,
    timeout: u16,
) -> Option<Bytes> {
    let client = UdpSocket::bind("127.0.0.1:0")
        .await
        .expect("Error bind client");

    client
        .send_to(&request_buf, server_address)
        .await
        .expect("Error on send request");

    let mut response = [0u8; 1024];
    match time::timeout(
        std::time::Duration::from_millis(timeout as u64),
        client.recv_from(&mut response),
    )
    .await
    {
        Ok(Ok((len, _))) => Some(Bytes::from(response[..len].to_vec())),
        _ => None,
    }
}
