use bytes::Bytes;
use quinn::{Endpoint, IdleTimeout, ServerConfig, TransportConfig, VarInt};
use rustls::pki_types::PrivateKeyDer;
use std::{
    array,
    net::{IpAddr, Ipv6Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};

#[tokio::main]
async fn main() {
    let mut transport_config = TransportConfig::default();

    let timeout = IdleTimeout::try_from(Duration::from_secs(100)).unwrap();
    transport_config.max_idle_timeout(Some(timeout));

    transport_config.max_concurrent_uni_streams(VarInt::from_u32(100000));

    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();

    // Save certificate for client to use
    std::fs::write("server_cert.der", cert.cert.der()).unwrap();

    let key = PrivateKeyDer::Pkcs8(cert.signing_key.serialize_der().into());

    let transport_config = Arc::new(transport_config);

    let mut server_config =
        ServerConfig::with_single_cert(vec![cert.cert.der().clone()], key).unwrap();

    server_config.transport_config(transport_config.clone());

    let mut roots = rustls::RootCertStore::empty();
    roots.add(cert.cert.der().clone()).unwrap();

    let addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 8080);

    dbg!(addr.to_string());

    let server = Endpoint::server(server_config, addr).unwrap();

    while let Some(connecting) = server.accept().await {
        let conn = connecting.await.unwrap();
        let mut chunks: [Bytes; 4] = array::from_fn(|_| Bytes::new());

        tokio::task::spawn(async move {
            loop {
                match conn.accept_uni().await {
                    Ok(mut recv_stream) => loop {
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
                    },
                    Err(_) => break,
                }
            }
        });
    }
}
