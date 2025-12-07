use std::{
    net::{IpAddr, Ipv6Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use quinn::{ClientConfig, Endpoint, IdleTimeout, TransportConfig, VarInt};

#[tokio::main]
async fn main() {
    let server_addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 8080);
    let addr = (std::net::Ipv6Addr::LOCALHOST, 0).into();

    let endpoint = Endpoint::client(addr).unwrap();

    let client = Arc::new(endpoint);

    let cert_der = std::fs::read("server_cert.der").unwrap();
    let mut roots = rustls::RootCertStore::empty();
    roots.add(cert_der.into()).unwrap();
    let mut client_config = ClientConfig::with_root_certificates(Arc::new(roots)).unwrap();

    let mut transport_config = TransportConfig::default();
    let timeout = IdleTimeout::try_from(Duration::from_secs(20)).unwrap();
    transport_config.max_concurrent_uni_streams(VarInt::from_u32(100000));
    transport_config.max_idle_timeout(Some(timeout));
    let transport_config = Arc::new(transport_config);
    client_config.transport_config(transport_config);

    let mut tasks = Vec::new();
    let streams = 100_000; // 10m streams

    for _ in 0..100 {
        let client = client.clone();
        let client_config = client_config.clone();

        let task = tokio::task::spawn(async move {
            let conn = client
                .connect_with(client_config, server_addr, "localhost")
                .unwrap()
                .await
                .unwrap();

            for _ in 0..streams {
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
