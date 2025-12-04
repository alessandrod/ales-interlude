use std::{
    net::{IpAddr, Ipv6Addr, SocketAddr},
    sync::Arc,
};

use quinn::{ClientConfig, Endpoint};

#[tokio::main]
async fn main() {
    let server_addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 8080);
    let addr = (std::net::Ipv6Addr::LOCALHOST, 0).into();

    let endpoint = Endpoint::client(addr).unwrap();

    let client = Arc::new(endpoint);

    let cert_der = std::fs::read("server_cert.der").unwrap();
    let mut roots = rustls::RootCertStore::empty();
    roots.add(cert_der.into()).unwrap();
    let client_config = ClientConfig::with_root_certificates(Arc::new(roots)).unwrap();

    let mut tasks = Vec::new();

    for _ in 0..1000 {
        let client = client.clone();
        let client_config = client_config.clone();

        let task = tokio::task::spawn(async move {
            let conn = client
                .connect_with(client_config, server_addr, "localhost")
                .unwrap()
                .await
                .unwrap();

            for _ in 0..1000 {
                let mut send_stream = conn.open_uni().await.unwrap();

                send_stream.write_all(&[0_u8; 1000]).await.unwrap();
            }
        });

        tasks.push(task);
    }
    let mut done: u32 = 0;
    for task in tasks {
        task.await.unwrap();

        done += 1;
        dbg!(done);
    }
}
