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
        Ok(_) => Some(Bytes::from(response)),
        Err(e) => panic!("Error on receive response: {}", e),
    }
}
