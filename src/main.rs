use bytes::Bytes;
use std::{
    net::{IpAddr, Ipv6Addr, SocketAddr},
    sync::Arc,
};

use quinn::{
    Endpoint, ServerConfig, TransportConfig, VarInt,
    rustls::{self, pki_types::PrivateKeyDer},
};

#[tokio::main]
async fn main() {
    let mut transport_config = TransportConfig::default();

    transport_config.max_concurrent_uni_streams(VarInt::from_u32(1000));

    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();

    let key = PrivateKeyDer::Pkcs8(cert.signing_key.serialize_der().into());

    let transport_config = Arc::new(transport_config);

    let mut server_config =
        ServerConfig::with_single_cert(vec![cert.cert.der().clone()], key).unwrap();

    server_config.transport_config(transport_config.clone());

    let mut roots = rustls::RootCertStore::empty();
    roots.add(cert.cert.der().clone()).unwrap();

    let addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 8080);

    let server = Endpoint::server(server_config, addr).unwrap();
    while let Some(connecting) = server.accept().await {
        let conn = connecting.await.unwrap();
        tokio::task::spawn(async move {
            loop {
                match conn.accept_uni().await {
                    Ok(mut recv_stream) => {
                        let mut chunks = vec![Bytes::new(); 10];

                        loop {
                            match recv_stream.read_chunks(&mut chunks).await {
                                Ok(n) => {
                                    if n.is_none() {
                                        break;
                                    }
                                }
                                Err(e) => {
                                    eprint!("{}", e);
                                    break;
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }

    let client = Arc::new(Endpoint::client((std::net::Ipv6Addr::LOCALHOST, 0).into()).unwrap());
    for _ in 0..1000 {
        let client = client.clone();
        tokio::task::spawn(async move {
            let conn = client.connect(addr, "localhost").unwrap().await.unwrap();

            for _ in 0..1000 {
                let mut send_stream = conn.open_uni().await.unwrap();
                send_stream.write_all(&[0u8; 1000]).await.unwrap();

                send_stream.finish().unwrap();

                _ = send_stream.stopped().await;
            }
        });
    }
}
