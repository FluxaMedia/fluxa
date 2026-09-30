use std::io::{ErrorKind, Read, Write};
use std::net::{IpAddr, TcpStream, ToSocketAddrs, UdpSocket};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

use mdns_sd::{ServiceDaemon, ServiceEvent};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, ClientConnection, DigitallySignedStruct, SignatureScheme, StreamOwned};
use serde_json::{Value, json};

use super::{Cmd, Device, Event, Protocol};
use crate::core_value;

const SSDP_ADDR: &str = "239.255.255.250:1900";
const SCAN: Duration = Duration::from_secs(4);
const CC_SENDER: &str = "sender-fluxa";
const CC_RECEIVER: &str = "receiver-0";
const CC_CONNECTION: &str = "urn:x-cast:com.google.cast.tp.connection";
const CC_HEARTBEAT: &str = "urn:x-cast:com.google.cast.tp.heartbeat";
const CC_RECEIVER_NS: &str = "urn:x-cast:com.google.cast.receiver";
const CC_MEDIA: &str = "urn:x-cast:com.google.cast.media";
const CC_APP: &str = "CC1AD845";
const FCAST_PORT: u16 = 46899;
const FCAST_PLAY: u64 = 1;
const FCAST_PAUSE: u64 = 2;
const FCAST_RESUME: u64 = 3;
const FCAST_STOP: u64 = 4;
const FCAST_SEEK: u64 = 5;
const FCAST_UPDATE: u64 = 6;
const FCAST_VERSION: u64 = 11;
const FCAST_PING: u64 = 12;
const FCAST_PONG: u64 = 13;
const DLNA_AVT: &str = "urn:schemas-upnp-org:service:AVTransport:1";

pub(super) fn lan_ip() -> Option<IpAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("1.1.1.1:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip())
}

pub(super) fn lan_url(url: &str) -> String {
    match lan_ip() {
        Some(ip) => core_value("castLanUrl", json!({"url": url, "lanIp": ip.to_string()}))
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .unwrap_or_else(|| url.to_owned()),
        None => url.to_owned(),
    }
}

fn split_url(url: &str) -> Option<(String, u16, String)> {
    let rest = url.strip_prefix("http://")?;
    let (authority, path) = rest
        .split_once('/')
        .map_or((rest, "/".to_owned()), |(a, p)| (a, format!("/{p}")));
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (host, port.parse().ok()?),
        None => (authority, 80),
    };
    Some((host.to_owned(), port, path))
}

fn http(
    method: &str,
    url: &str,
    headers: &[(&str, String)],
    body: &str,
) -> Result<(u16, String), String> {
    let (host, port, path) = split_url(url).ok_or("unsupported url")?;
    let address = (host.as_str(), port)
        .to_socket_addrs()
        .map_err(|e| e.to_string())?
        .next()
        .ok_or("unresolved host")?;
    let mut stream =
        TcpStream::connect_timeout(&address, Duration::from_secs(4)).map_err(|e| e.to_string())?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
    let mut request = format!(
        "{method} {path} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\nContent-Length: {}\r\n",
        body.len()
    );
    for (name, value) in headers {
        request.push_str(&format!("{name}: {value}\r\n"));
    }
    request.push_str("\r\n");
    request.push_str(body);
    stream
        .write_all(request.as_bytes())
        .map_err(|e| e.to_string())?;
    let mut raw = Vec::new();
    let _ = stream.read_to_end(&mut raw);
    let text = String::from_utf8_lossy(&raw).into_owned();
    let (head, body) = text.split_once("\r\n\r\n").ok_or("malformed response")?;
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse().ok())
        .ok_or("malformed response")?;
    let chunked = head
        .to_ascii_lowercase()
        .contains("transfer-encoding: chunked");
    Ok((
        status,
        if chunked {
            dechunk(body)
        } else {
            body.to_owned()
        },
    ))
}

fn dechunk(body: &str) -> String {
    let mut out = String::new();
    let mut rest = body;
    while let Some((size, tail)) = rest.split_once("\r\n") {
        let Ok(size) = usize::from_str_radix(size.trim(), 16) else {
            break;
        };
        if size == 0 || tail.len() < size {
            break;
        }
        out.push_str(&tail[..size]);
        rest = tail[size..].trim_start_matches("\r\n");
    }
    out
}

fn http_ok(
    method: &str,
    url: &str,
    headers: &[(&str, String)],
    body: &str,
) -> Result<String, String> {
    let (status, body) = http(method, url, headers, body)?;
    if (200..300).contains(&status) {
        Ok(body)
    } else {
        Err(format!("device answered {status}"))
    }
}

pub(super) fn discover(tx: Sender<Device>) {
    let ssdp = tx.clone();
    std::thread::spawn(move || ssdp_scan(ssdp));
    std::thread::spawn(move || mdns_scan(tx));
}

fn ssdp_scan(tx: Sender<Device>) {
    let Ok(socket) = UdpSocket::bind("0.0.0.0:0") else {
        return;
    };
    socket
        .set_read_timeout(Some(Duration::from_millis(300)))
        .ok();
    for target in [DLNA_AVT, "roku:ecp"] {
        let search = format!(
            "M-SEARCH * HTTP/1.1\r\nHOST: {SSDP_ADDR}\r\nMAN: \"ssdp:discover\"\r\nMX: 2\r\nST: {target}\r\n\r\n"
        );
        let _ = socket.send_to(search.as_bytes(), SSDP_ADDR);
    }
    let mut seen: Vec<String> = Vec::new();
    let mut buf = [0u8; 2048];
    let deadline = Instant::now() + SCAN;
    while Instant::now() < deadline {
        let Ok((len, _)) = socket.recv_from(&mut buf) else {
            continue;
        };
        let response = String::from_utf8_lossy(&buf[..len]).into_owned();
        let header = |name: &str| {
            response
                .lines()
                .find(|line| line.to_ascii_uppercase().starts_with(name))
                .and_then(|line| line.split_once(':'))
                .map(|(_, value)| value.trim().to_owned())
        };
        let Some(location) = header("LOCATION:") else {
            continue;
        };
        if seen.contains(&location) {
            continue;
        }
        seen.push(location.clone());
        let tx = tx.clone();
        let roku = header("ST:").is_some_and(|target| target.contains("roku"));
        std::thread::spawn(move || {
            let device = if roku {
                roku_device(&location)
            } else {
                dlna_device(&location)
            };
            if let Some(device) = device {
                let _ = tx.send(device);
            }
        });
    }
}

fn dlna_device(location: &str) -> Option<Device> {
    let xml = http_ok("GET", location, &[], "").ok()?;
    let parsed = core_value("castDlnaDevice", json!({"xml": xml, "baseUrl": location}))?;
    Some(Device {
        id: location.to_owned(),
        name: parsed["name"].as_str()?.to_owned(),
        protocol: Protocol::Dlna,
        host: String::new(),
        port: 0,
        control_url: parsed["controlUrl"].as_str().map(ToOwned::to_owned),
    })
}

fn roku_device(location: &str) -> Option<Device> {
    let (host, _, _) = split_url(location)?;
    let info = http_ok(
        "GET",
        &format!("http://{host}:8060/query/device-info"),
        &[],
        "",
    )
    .ok();
    let name = info
        .and_then(|xml| core_value("castRokuName", json!({"xml": xml})))
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "Roku".to_owned());
    Some(Device {
        id: host.clone(),
        name,
        protocol: Protocol::Roku,
        host,
        port: 8060,
        control_url: None,
    })
}

fn mdns_scan(tx: Sender<Device>) {
    let Ok(daemon) = ServiceDaemon::new() else {
        return;
    };
    let kinds = [
        ("_googlecast._tcp.local.", Protocol::Chromecast),
        ("_airplay._tcp.local.", Protocol::Airplay),
        ("_fcast._tcp.local.", Protocol::Fcast),
    ];
    let receivers: Vec<_> = kinds
        .iter()
        .filter_map(|(service, protocol)| Some((daemon.browse(service).ok()?, *protocol)))
        .collect();
    let deadline = Instant::now() + SCAN;
    while Instant::now() < deadline {
        for (receiver, protocol) in &receivers {
            let Ok(ServiceEvent::ServiceResolved(info)) =
                receiver.recv_timeout(Duration::from_millis(100))
            else {
                continue;
            };
            let Some(address) = info.get_addresses().iter().next() else {
                continue;
            };
            let host = address.to_string();
            let port = match (info.get_port(), protocol) {
                (0, Protocol::Fcast) => FCAST_PORT,
                (port, _) => port,
            };
            let property = match protocol {
                Protocol::Chromecast => "fn",
                _ => "name",
            };
            let name = info
                .get_property_val_str(property)
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| {
                    info.get_fullname()
                        .split('.')
                        .next()
                        .unwrap_or("")
                        .to_owned()
                });
            let _ = tx.send(Device {
                id: format!("{host}:{port}"),
                name,
                protocol: *protocol,
                host,
                port,
                control_url: None,
            });
        }
    }
    let _ = daemon.shutdown();
}

pub(super) fn start(
    device: Device,
    media_url: String,
    title: String,
    resume: f64,
    events: Sender<Event>,
    commands: Receiver<Cmd>,
) {
    std::thread::spawn(move || {
        let result = match device.protocol {
            Protocol::Dlna => dlna(&device, &media_url, &title, &events, &commands),
            Protocol::Roku => roku(&device, &media_url, &events, &commands),
            Protocol::Airplay => airplay(&device, &media_url, resume, &events, &commands),
            Protocol::Fcast => fcast(&device, &media_url, resume, &events, &commands),
            Protocol::Chromecast => {
                chromecast(&device, &media_url, &title, resume, &events, &commands)
            }
        };
        if let Err(error) = result {
            let _ = events.send(Event::Failed(error));
        }
    });
}

fn plan(route: &str, args: Value) -> Result<String, String> {
    core_value(route, args)
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .ok_or_else(|| "unsupported media url".to_owned())
}

fn dlna_action(url: &str, urn: &str, action: &str, args: &str) -> Result<(), String> {
    let body = plan(
        "castDlnaSoap",
        json!({"urn": urn, "action": action, "args": args}),
    )?;
    http_ok(
        "POST",
        url,
        &[
            ("Content-Type", "text/xml; charset=\"utf-8\"".to_owned()),
            ("SOAPACTION", format!("\"{urn}#{action}\"")),
        ],
        &body,
    )
    .map(drop)
}

fn dlna(
    device: &Device,
    media_url: &str,
    title: &str,
    events: &Sender<Event>,
    commands: &Receiver<Cmd>,
) -> Result<(), String> {
    let control = device
        .control_url
        .clone()
        .ok_or("device has no transport")?;
    let args = plan(
        "castDlnaLoadArgs",
        json!({"mediaUrl": media_url, "title": title}),
    )?;
    dlna_action(&control, DLNA_AVT, "SetAVTransportURI", &args)?;
    dlna_action(
        &control,
        DLNA_AVT,
        "Play",
        "<InstanceID>0</InstanceID><Speed>1</Speed>",
    )?;
    let _ = events.send(Event::Ready);
    while let Ok(command) = commands.recv() {
        let result = match command {
            Cmd::Pause => dlna_action(&control, DLNA_AVT, "Pause", "<InstanceID>0</InstanceID>"),
            Cmd::Resume => dlna_action(
                &control,
                DLNA_AVT,
                "Play",
                "<InstanceID>0</InstanceID><Speed>1</Speed>",
            ),
            Cmd::Seek(position) => plan("castDlnaSeekArgs", json!({"position": position}))
                .and_then(|args| dlna_action(&control, DLNA_AVT, "Seek", &args)),
            Cmd::Stop => break,
        };
        if let Err(error) = result {
            let _ = events.send(Event::Failed(error));
        }
    }
    let _ = dlna_action(&control, DLNA_AVT, "Stop", "<InstanceID>0</InstanceID>");
    Ok(())
}

fn roku(
    device: &Device,
    media_url: &str,
    events: &Sender<Event>,
    commands: &Receiver<Cmd>,
) -> Result<(), String> {
    let url = plan(
        "castRokuLaunch",
        json!({"host": device.host, "mediaUrl": media_url}),
    )?;
    http_ok("POST", &url, &[], "")?;
    let _ = events.send(Event::Ready);
    let key = |name: &str| {
        http_ok(
            "POST",
            &format!("http://{}:8060/keypress/{name}", device.host),
            &[],
            "",
        )
    };
    while let Ok(command) = commands.recv() {
        let result = match command {
            Cmd::Pause | Cmd::Resume => key("Play"),
            Cmd::Stop => break,
            Cmd::Seek(_) => Ok(String::new()),
        };
        if let Err(error) = result {
            let _ = events.send(Event::Failed(error));
        }
    }
    let _ = key("Home");
    Ok(())
}

fn airplay(
    device: &Device,
    media_url: &str,
    resume: f64,
    events: &Sender<Event>,
    commands: &Receiver<Cmd>,
) -> Result<(), String> {
    let base = format!("http://{}:{}", device.host, device.port);
    let body = plan("castAirplayBody", json!({"mediaUrl": media_url}))?;
    http_ok(
        "PUT",
        &format!("{base}/play"),
        &[("Content-Type", "text/parameters".to_owned())],
        &body,
    )?;
    if resume > 1.0 {
        let _ = http_ok(
            "POST",
            &format!("{base}/scrub?position={resume:.3}"),
            &[],
            "",
        );
    }
    let _ = events.send(Event::Ready);
    while let Ok(command) = commands.recv() {
        let url = match command {
            Cmd::Pause => format!("{base}/rate?value=0.000000"),
            Cmd::Resume => format!("{base}/rate?value=1.000000"),
            Cmd::Seek(position) => format!("{base}/scrub?position={position:.3}"),
            Cmd::Stop => break,
        };
        if let Err(error) = http_ok("POST", &url, &[], "") {
            let _ = events.send(Event::Failed(error));
        }
    }
    let _ = http_ok("POST", &format!("{base}/stop"), &[], "");
    Ok(())
}

trait Wire: Read + Write {
    fn tcp(&self) -> &TcpStream;
}

impl Wire for TcpStream {
    fn tcp(&self) -> &TcpStream {
        self
    }
}

impl Wire for StreamOwned<ClientConnection, TcpStream> {
    fn tcp(&self) -> &TcpStream {
        &self.sock
    }
}

struct Frames<S: Wire> {
    stream: S,
    buffer: Vec<u8>,
    little_endian: bool,
}

impl<S: Wire> Frames<S> {
    fn new(stream: S, little_endian: bool) -> Self {
        stream
            .tcp()
            .set_read_timeout(Some(Duration::from_millis(250)))
            .ok();
        stream
            .tcp()
            .set_write_timeout(Some(Duration::from_secs(5)))
            .ok();
        Self {
            stream,
            buffer: Vec::new(),
            little_endian,
        }
    }

    fn write(&mut self, payload: &[u8]) -> Result<(), String> {
        let size = payload.len() as u32;
        let header = if self.little_endian {
            size.to_le_bytes()
        } else {
            size.to_be_bytes()
        };
        self.stream.write_all(&header).map_err(|e| e.to_string())?;
        self.stream.write_all(payload).map_err(|e| e.to_string())?;
        self.stream.flush().map_err(|e| e.to_string())
    }

    fn read(&mut self) -> Result<Option<Vec<u8>>, String> {
        let mut chunk = [0u8; 4096];
        match self.stream.read(&mut chunk) {
            Ok(0) => return Err("device closed the connection".to_owned()),
            Ok(n) => self.buffer.extend_from_slice(&chunk[..n]),
            Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
            Err(e) => return Err(e.to_string()),
        }
        if self.buffer.len() < 4 {
            return Ok(None);
        }
        let header = [
            self.buffer[0],
            self.buffer[1],
            self.buffer[2],
            self.buffer[3],
        ];
        let size = if self.little_endian {
            u32::from_le_bytes(header)
        } else {
            u32::from_be_bytes(header)
        } as usize;
        if size > 1 << 20 {
            return Err("oversized frame".to_owned());
        }
        if self.buffer.len() < 4 + size {
            return Ok(None);
        }
        let frame = self.buffer[4..4 + size].to_vec();
        self.buffer.drain(..4 + size);
        Ok(Some(frame))
    }
}

fn number_array(bytes: &[u8]) -> Value {
    Value::Array(bytes.iter().map(|byte| json!(byte)).collect())
}

fn byte_vec(value: Value) -> Result<Vec<u8>, String> {
    Ok(value
        .as_array()
        .ok_or("message rejected")?
        .iter()
        .filter_map(|byte| byte.as_u64().map(|byte| byte as u8))
        .collect())
}

struct Fcast {
    frames: Frames<TcpStream>,
}

impl Fcast {
    fn send(&mut self, opcode: u64, body: Option<String>) -> Result<(), String> {
        let message = core_value("castFcastEncode", json!({"opcode": opcode, "body": body}))
            .ok_or("message too large")?;
        self.frames.write(&byte_vec(message)?)
    }

    fn poll(&mut self) -> Result<Option<(u64, String)>, String> {
        let Some(frame) = self.frames.read()? else {
            return Ok(None);
        };
        let decoded = core_value("castFcastDecode", json!({"bytes": number_array(&frame)}));
        Ok(decoded.and_then(|value| {
            Some((
                value["opcode"].as_u64()?,
                value["body"].as_str()?.to_owned(),
            ))
        }))
    }
}

fn fcast(
    device: &Device,
    media_url: &str,
    resume: f64,
    events: &Sender<Event>,
    commands: &Receiver<Cmd>,
) -> Result<(), String> {
    let play = plan(
        "castFcastPlayBody",
        json!({"mediaUrl": media_url, "resume": resume}),
    )?;
    let version = plan("castFcastVersionBody", json!({"version": 2}))?;
    let address = (device.host.as_str(), device.port)
        .to_socket_addrs()
        .map_err(|e| e.to_string())?
        .next()
        .ok_or("unresolved host")?;
    let stream =
        TcpStream::connect_timeout(&address, Duration::from_secs(4)).map_err(|e| e.to_string())?;
    let mut session = Fcast {
        frames: Frames::new(stream, true),
    };
    session.send(FCAST_VERSION, Some(version))?;
    session.send(FCAST_PLAY, Some(play))?;
    let _ = events.send(Event::Ready);
    loop {
        if let Some((opcode, body)) = session.poll()? {
            if opcode == FCAST_PING {
                session.send(FCAST_PONG, None)?;
            } else if opcode == FCAST_UPDATE
                && let Some(update) = core_value("castFcastUpdate", json!({"body": body}))
            {
                let _ = events.send(Event::Position {
                    seconds: update["time"].as_f64().unwrap_or(0.0),
                    playing: update["state"].as_u64() == Some(1),
                });
            }
        }
        match commands.recv_timeout(Duration::from_millis(1)) {
            Ok(Cmd::Pause) => session.send(FCAST_PAUSE, None)?,
            Ok(Cmd::Resume) => session.send(FCAST_RESUME, None)?,
            Ok(Cmd::Seek(position)) => {
                let body = plan("castFcastSeekBody", json!({"position": position}))?;
                session.send(FCAST_SEEK, Some(body))?
            }
            Ok(Cmd::Stop) | Err(RecvTimeoutError::Disconnected) => {
                let _ = session.send(FCAST_STOP, None);
                return Ok(());
            }
            Err(RecvTimeoutError::Timeout) => {}
        }
    }
}

#[derive(Debug)]
struct AcceptAny;

impl ServerCertVerifier for AcceptAny {
    fn verify_server_cert(
        &self,
        _: &CertificateDer<'_>,
        _: &[CertificateDer<'_>],
        _: &ServerName<'_>,
        _: &[u8],
        _: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

struct Chromecast {
    frames: Frames<StreamOwned<ClientConnection, TcpStream>>,
    request: i64,
}

impl Chromecast {
    fn send(&mut self, destination: &str, namespace: &str, payload: Value) -> Result<(), String> {
        let message = core_value(
            "castChromecastEncode",
            json!({
                "source": CC_SENDER,
                "destination": destination,
                "namespace": namespace,
                "payload": payload.to_string(),
            }),
        )
        .ok_or("message rejected")?;
        self.frames.write(&byte_vec(message)?)
    }

    fn request(
        &mut self,
        destination: &str,
        namespace: &str,
        mut payload: Value,
    ) -> Result<(), String> {
        self.request += 1;
        payload["requestId"] = json!(self.request);
        self.send(destination, namespace, payload)
    }

    fn poll(&mut self) -> Result<Option<(String, Value)>, String> {
        let Some(frame) = self.frames.read()? else {
            return Ok(None);
        };
        let Some(decoded) = core_value(
            "castChromecastDecode",
            json!({"bytes": number_array(&frame)}),
        ) else {
            return Ok(None);
        };
        let namespace = decoded["namespace"].as_str().unwrap_or_default().to_owned();
        let payload = serde_json::from_str(decoded["payload"].as_str().unwrap_or("{}"))
            .unwrap_or(Value::Null);
        if namespace == CC_HEARTBEAT && payload["type"] == "PING" {
            self.send(CC_RECEIVER, CC_HEARTBEAT, json!({"type": "PONG"}))?;
        }
        Ok(Some((namespace, payload)))
    }

    fn wait(&mut self, namespace: &str, matches: impl Fn(&Value) -> bool) -> Result<Value, String> {
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            if let Some((received, payload)) = self.poll()?
                && received == namespace
                && matches(&payload)
            {
                return Ok(payload);
            }
        }
        Err("device did not answer".to_owned())
    }
}

fn chromecast(
    device: &Device,
    media_url: &str,
    title: &str,
    resume: f64,
    events: &Sender<Event>,
    commands: &Receiver<Cmd>,
) -> Result<(), String> {
    let valid =
        core_value("castValidUrl", json!({"url": media_url})).and_then(|value| value.as_bool());
    if valid != Some(true) {
        return Err("unsupported media url".to_owned());
    }
    let content_type = plan("castContentType", json!({"url": media_url}))?;
    let address = (device.host.as_str(), device.port)
        .to_socket_addrs()
        .map_err(|e| e.to_string())?
        .next()
        .ok_or("unresolved host")?;
    let tcp =
        TcpStream::connect_timeout(&address, Duration::from_secs(4)).map_err(|e| e.to_string())?;
    let config =
        ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .map_err(|e| e.to_string())?
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(AcceptAny))
            .with_no_client_auth();
    let name = ServerName::try_from(device.host.clone()).map_err(|_| "invalid host".to_owned())?;
    let connection = ClientConnection::new(Arc::new(config), name).map_err(|e| e.to_string())?;
    let mut cast = Chromecast {
        frames: Frames::new(StreamOwned::new(connection, tcp), false),
        request: 0,
    };
    cast.send(CC_RECEIVER, CC_CONNECTION, json!({"type": "CONNECT"}))?;
    cast.request(
        CC_RECEIVER,
        CC_RECEIVER_NS,
        json!({"type": "LAUNCH", "appId": CC_APP}),
    )?;
    let status = cast.wait(CC_RECEIVER_NS, |payload| {
        payload["status"]["applications"]
            .as_array()
            .is_some_and(|apps| apps.iter().any(|app| app["appId"] == CC_APP))
    })?;
    let transport = status["status"]["applications"]
        .as_array()
        .and_then(|apps| apps.iter().find(|app| app["appId"] == CC_APP))
        .and_then(|app| app["transportId"].as_str())
        .ok_or("device returned no transport")?
        .to_owned();
    cast.send(&transport, CC_CONNECTION, json!({"type": "CONNECT"}))?;
    cast.request(
        &transport,
        CC_MEDIA,
        json!({
            "type": "LOAD",
            "media": {"contentId": media_url, "contentType": content_type, "streamType": "BUFFERED"},
            "currentTime": resume,
            "autoplay": true,
            "customData": {"title": title},
        }),
    )?;
    let loaded = cast.wait(CC_MEDIA, |payload| {
        payload["status"][0]["mediaSessionId"].is_number()
    })?;
    let mut session_id = loaded["status"][0]["mediaSessionId"].as_i64().unwrap_or(0);
    let _ = events.send(Event::Ready);
    let mut ping = Instant::now();
    loop {
        if let Some((namespace, payload)) = cast.poll()?
            && namespace == CC_MEDIA
            && let Some(status) = payload["status"].get(0)
        {
            if let Some(id) = status["mediaSessionId"].as_i64() {
                session_id = id;
            }
            if let Some(seconds) = status["currentTime"].as_f64() {
                let _ = events.send(Event::Position {
                    seconds,
                    playing: status["playerState"] == "PLAYING",
                });
            }
        }
        if ping.elapsed() > Duration::from_secs(5) {
            cast.send(CC_RECEIVER, CC_HEARTBEAT, json!({"type": "PING"}))?;
            ping = Instant::now();
        }
        match commands.recv_timeout(Duration::from_millis(1)) {
            Ok(Cmd::Pause) => cast.request(
                &transport,
                CC_MEDIA,
                json!({"type": "PAUSE", "mediaSessionId": session_id}),
            )?,
            Ok(Cmd::Resume) => cast.request(
                &transport,
                CC_MEDIA,
                json!({"type": "PLAY", "mediaSessionId": session_id}),
            )?,
            Ok(Cmd::Seek(position)) => cast.request(
                &transport,
                CC_MEDIA,
                json!({"type": "SEEK", "mediaSessionId": session_id, "currentTime": position}),
            )?,
            Ok(Cmd::Stop) | Err(RecvTimeoutError::Disconnected) => {
                let _ = cast.request(
                    &transport,
                    CC_MEDIA,
                    json!({"type": "STOP", "mediaSessionId": session_id}),
                );
                return Ok(());
            }
            Err(RecvTimeoutError::Timeout) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::TcpListener;
    use std::sync::mpsc::channel;

    use super::*;

    fn device(protocol: Protocol, port: u16) -> Device {
        Device {
            id: "test".to_owned(),
            name: "test".to_owned(),
            protocol,
            host: "127.0.0.1".to_owned(),
            port,
            control_url: None,
        }
    }

    #[test]
    fn fcast_sends_version_then_play_with_the_media_url() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut frames = Frames::new(stream, true);
            let mut received = Vec::new();
            while received.len() < 2 {
                if let Some(frame) = frames.read().unwrap() {
                    received.push(frame);
                }
            }
            received
        });
        let (events, ready) = channel();
        let (commands, inbox) = channel();
        start(
            device(Protocol::Fcast, port),
            "http://192.168.1.5/movie.mp4".to_owned(),
            "Movie".to_owned(),
            0.0,
            events,
            inbox,
        );
        assert!(matches!(
            ready.recv_timeout(Duration::from_secs(5)),
            Ok(Event::Ready)
        ));
        let received = server.join().unwrap();
        assert_eq!(received[0][0], FCAST_VERSION as u8);
        assert_eq!(received[1][0], FCAST_PLAY as u8);
        assert!(String::from_utf8_lossy(&received[1][1..]).contains("movie.mp4"));
        drop(commands);
    }

    #[test]
    fn urls_split_into_host_port_and_path() {
        let (url, path) = split_url("http://10.0.0.2:8060/launch/2213?t=v")
            .map(|(h, p, path)| (format!("{h}:{p}"), path))
            .unwrap();
        assert_eq!(url, "10.0.0.2:8060");
        assert_eq!(path, "/launch/2213?t=v");
    }

    #[test]
    fn chunked_bodies_are_reassembled() {
        assert_eq!(
            dechunk("5\r\nhello\r\n6\r\n world\r\n0\r\n\r\n"),
            "hello world"
        );
    }
}
