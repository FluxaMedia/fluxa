use super::world::{Recorded, World};
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::TlsAcceptor;

pub const CA_PEM: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/scenarios/support/tls/ca.pem"
);
const SERVER_CERT: &[u8] = include_bytes!("tls/server.pem");
const SERVER_KEY: &[u8] = include_bytes!("tls/server.pk8.pem");

pub fn spawn(world: Arc<World>) -> u16 {
    let (port_tx, port_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("proxy runtime");
        runtime.block_on(async move {
            let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind proxy");
            port_tx
                .send(listener.local_addr().expect("proxy address").port())
                .ok();
            let acceptor = acceptor();
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    continue;
                };
                tokio::spawn(connection(stream, acceptor.clone(), world.clone()));
            }
        });
    });
    port_rx.recv().expect("proxy port")
}

fn acceptor() -> TlsAcceptor {
    let certs: Vec<CertificateDer<'static>> = CertificateDer::pem_slice_iter(SERVER_CERT)
        .collect::<Result<_, _>>()
        .expect("server certificate");
    let key = PrivateKeyDer::from_pem_slice(SERVER_KEY).expect("server key");
    let config = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .expect("protocol versions")
    .with_no_client_auth()
    .with_single_cert(certs, key)
    .expect("server config");
    TlsAcceptor::from(Arc::new(config))
}

async fn connection(mut stream: TcpStream, acceptor: TlsAcceptor, world: Arc<World>) {
    let mut head = Vec::new();
    let mut byte = [0u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        match stream.read(&mut byte).await {
            Ok(1) => head.push(byte[0]),
            _ => return,
        }
    }
    let head = String::from_utf8_lossy(&head).into_owned();
    let mut first = head.lines().next().unwrap_or("").split_whitespace();
    if first.next() != Some("CONNECT") {
        let _ = stream
            .write_all(b"HTTP/1.1 501 Not Implemented\r\ncontent-length: 0\r\n\r\n")
            .await;
        return;
    }
    let host = first
        .next()
        .and_then(|target| target.rsplit_once(':').map(|(host, _)| host.to_owned()))
        .unwrap_or_default();
    if stream
        .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
        .await
        .is_err()
    {
        return;
    }
    let Ok(tls) = acceptor.accept(stream).await else {
        return;
    };
    let service = service_fn(move |request: Request<hyper::body::Incoming>| {
        let world = world.clone();
        let host = host.clone();
        async move { Ok::<_, Infallible>(respond(&world, &host, request).await) }
    });
    let _ = hyper::server::conn::http1::Builder::new()
        .serve_connection(TokioIo::new(tls), service)
        .await;
}

async fn respond(
    world: &World,
    host: &str,
    request: Request<hyper::body::Incoming>,
) -> Response<Full<Bytes>> {
    let (parts, body) = request.into_parts();
    let body = body
        .collect()
        .await
        .map(|collected| collected.to_bytes())
        .unwrap_or_default();
    let recorded = Recorded {
        host: host.to_owned(),
        method: parts.method.to_string(),
        path: decode(parts.uri.path()),
        query: parts.uri.query().unwrap_or("").to_owned(),
        headers: parts
            .headers
            .iter()
            .map(|(name, value)| {
                (
                    name.to_string(),
                    String::from_utf8_lossy(value.as_bytes()).into_owned(),
                )
            })
            .collect(),
        body: String::from_utf8_lossy(&body).into_owned(),
    };
    let reply = world.handle(recorded);
    Response::builder()
        .status(reply.status)
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(reply.body)))
        .unwrap_or_default()
}

fn decode(path: &str) -> String {
    let bytes = path.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = bytes
            .get(i + 1..i + 3)
            .and_then(|pair| std::str::from_utf8(pair).ok());
        match (
            bytes[i],
            hex.and_then(|pair| u8::from_str_radix(pair, 16).ok()),
        ) {
            (b'%', Some(byte)) => {
                out.push(byte);
                i += 3;
            }
            (byte, _) => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}
