use bytes::Bytes;
use tokio::net::UdpSocket;

pub async fn get_free_port() -> u16 {
    let listener = UdpSocket::bind("127.0.0.1:0")
        .await
        .expect("Não foi possível encontrar uma porta livre");

    let port = listener
        .local_addr()
        .expect("Falha ao ler endereço local")
        .port();

    port
}

pub async fn send_and_receive(server_address: &str, request_buf: Bytes) -> Bytes {
    let client = UdpSocket::bind("127.0.0.1:0")
        .await
        .expect("Error bind client");

    client
        .send_to(&request_buf, server_address)
        .await
        .expect("Error on send request");

    let mut response = [0u8; 1024];
    let (len, _) = client
        .recv_from(&mut response)
        .await
        .expect("Error receive response");

    Bytes::from(response[..len].to_vec())
}
