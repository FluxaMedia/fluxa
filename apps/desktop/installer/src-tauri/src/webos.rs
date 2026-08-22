use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use russh::client::{self, Config, Handler};
use russh::keys::{decode_secret_key, PrivateKeyWithHashAlg};
use serde::Serialize;
use tokio::io::AsyncWriteExt;
use tokio::net::UdpSocket;

const SSH_PORT: u16 = 9922;
const SSH_USER: &str = "prisoner";
const KEY_PORT: u16 = 9991;
const TEMP_DIR: &str = "/media/developer/temp";
const SSDP_ADDR: &str = "239.255.255.250:1900";

#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("could not reach the TV at {host}: {source}")]
    Unreachable {
        host: String,
        #[source]
        source: std::io::Error,
    },
    #[error("the TV did not hand out a developer key. Is Developer Mode running with key server enabled?")]
    NoDeveloperKey,
    #[error("the passphrase did not unlock the developer key")]
    BadPassphrase,
    #[error("the TV rejected the developer key. It may have expired, restart the Developer Mode session")]
    Rejected,
    #[error("{0}")]
    Other(String),
}

type Result<T> = std::result::Result<T, InstallError>;

impl From<russh::Error> for InstallError {
    fn from(value: russh::Error) -> Self {
        InstallError::Other(value.to_string())
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredTv {
    pub host: String,
    pub name: String,
}

struct AcceptAll;

impl Handler for AcceptAll {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _key: &russh::keys::PublicKey,
    ) -> std::result::Result<bool, Self::Error> {
        Ok(true)
    }
}

pub async fn discover(timeout: Duration) -> Vec<DiscoveredTv> {
    let mut found: Vec<DiscoveredTv> = Vec::new();
    let Ok(socket) = UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0))).await else {
        return found;
    };
    let probe = format!(
        "M-SEARCH * HTTP/1.1\r\nHOST: {SSDP_ADDR}\r\nMAN: \"ssdp:discover\"\r\nMX: 2\r\nST: urn:lge-com:service:webos-second-screen:1\r\n\r\n"
    );
    if socket.send_to(probe.as_bytes(), SSDP_ADDR).await.is_err() {
        return found;
    }

    let deadline = tokio::time::Instant::now() + timeout;
    let mut buffer = vec![0u8; 2048];
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        let Ok(Ok((size, from))) =
            tokio::time::timeout(remaining, socket.recv_from(&mut buffer)).await
        else {
            break;
        };
        let host = from.ip().to_string();
        if found.iter().any(|tv| tv.host == host) {
            continue;
        }
        let body = String::from_utf8_lossy(&buffer[..size]);
        let name = body
            .lines()
            .find_map(|line| line.strip_prefix("DLNADeviceName.lge.com:"))
            .map(|value| percent_decode(value.trim()))
            .unwrap_or_else(|| "LG webOS TV".to_string());
        found.push(DiscoveredTv { host, name });
    }
    found
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&input[index + 1..index + 3], 16) {
                out.push(byte);
                index += 3;
                continue;
            }
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub async fn fetch_developer_key(host: &str) -> Result<String> {
    let url = format!("http://{host}:{KEY_PORT}/webos_rsa");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|err| InstallError::Other(err.to_string()))?;
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|_| InstallError::NoDeveloperKey)?;
    if !response.status().is_success() {
        return Err(InstallError::NoDeveloperKey);
    }
    let body = response
        .text()
        .await
        .map_err(|err| InstallError::Other(err.to_string()))?;
    if !body.contains("PRIVATE KEY") {
        return Err(InstallError::NoDeveloperKey);
    }
    Ok(body)
}

pub struct Session {
    handle: client::Handle<AcceptAll>,
}

impl Session {
    pub async fn connect(host: &str, passphrase: &str, key_pem: &str) -> Result<Self> {
        let key = decode_secret_key(key_pem, Some(passphrase))
            .map_err(|_| InstallError::BadPassphrase)?;
        let address: IpAddr = host
            .parse()
            .map_err(|_| InstallError::Other(format!("{host} is not a valid IP address")))?;

        let config = Arc::new(Config {
            inactivity_timeout: Some(Duration::from_secs(60)),
            ..Config::default()
        });
        let mut handle = client::connect(config, SocketAddr::new(address, SSH_PORT), AcceptAll)
            .await
            .map_err(|err| InstallError::Unreachable {
                host: host.to_string(),
                source: std::io::Error::other(err.to_string()),
            })?;

        let authenticated = handle
            .authenticate_publickey(
                SSH_USER,
                PrivateKeyWithHashAlg::new(Arc::new(key), None),
            )
            .await?;
        if !authenticated.success() {
            return Err(InstallError::Rejected);
        }
        Ok(Self { handle })
    }

    pub async fn exec(&self, command: &str) -> Result<String> {
        let mut channel = self.handle.channel_open_session().await?;
        channel.exec(true, command).await?;
        let mut output = Vec::new();
        while let Some(message) = channel.wait().await {
            match message {
                russh::ChannelMsg::Data { ref data } => output.extend_from_slice(data),
                russh::ChannelMsg::ExtendedData { ref data, .. } => output.extend_from_slice(data),
                russh::ChannelMsg::Eof | russh::ChannelMsg::Close => break,
                _ => {}
            }
        }
        Ok(String::from_utf8_lossy(&output).into_owned())
    }

    pub async fn upload(&self, bytes: &[u8], remote_path: &str) -> Result<()> {
        let mut channel = self.handle.channel_open_session().await?;
        channel
            .exec(true, format!("cat > {remote_path}").as_str())
            .await?;
        let mut writer = channel.make_writer();
        writer
            .write_all(bytes)
            .await
            .map_err(|err| InstallError::Other(err.to_string()))?;
        writer
            .shutdown()
            .await
            .map_err(|err| InstallError::Other(err.to_string()))?;
        while let Some(message) = channel.wait().await {
            if matches!(
                message,
                russh::ChannelMsg::Eof | russh::ChannelMsg::Close | russh::ChannelMsg::ExitStatus { .. }
            ) {
                break;
            }
        }
        Ok(())
    }
}

pub fn remote_ipk_path(file_name: &str) -> String {
    format!("{TEMP_DIR}/{file_name}")
}

pub fn install_command(remote_path: &str) -> String {
    let payload = serde_json::json!({
        "id": "com.ares.defaultName",
        "ipkUrl": remote_path,
        "subscribe": true,
    });
    format!(
        "/usr/bin/luna-send-pub -i -f luna://com.webos.appInstallService/dev/install '{payload}'"
    )
}

pub fn launch_command(app_id: &str) -> String {
    let payload = serde_json::json!({ "id": app_id });
    format!(
        "/usr/bin/luna-send-pub -n 1 -f luna://com.webos.applicationManager/launch '{payload}'"
    )
}

pub fn install_failed_reason(log: &str) -> Option<String> {
    let compact = log.replace(char::is_whitespace, "");
    if compact.contains("\"state\":\"installed\"") {
        return None;
    }
    if let Some(text) = log
        .lines()
        .find_map(|line| line.split("\"errorText\":\"").nth(1))
        .and_then(|rest| rest.split('"').next())
    {
        return Some(text.to_string());
    }
    if compact.contains("\"state\":\"installfailed\"") || compact.contains("\"returnValue\":false") {
        return Some("the TV rejected the package".to_string());
    }
    Some("the TV did not report a successful install".to_string())
}

pub fn ensure_temp_dir_command() -> String {
    format!("/usr/bin/test -d {TEMP_DIR} || /bin/mkdir -p {TEMP_DIR}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installed_state_anywhere_in_the_stream_counts_as_success() {
        let log = r#"{"returnValue":true,"details":{"state":"installing"}}
{"returnValue":true,"details":{"state":"installed"}}"#;
        assert_eq!(install_failed_reason(log), None);
    }

    #[test]
    fn surfaces_the_error_text_the_tv_sent() {
        let log = r#"{"returnValue":false,"errorText":"Package is not signed","errorCode":-1}"#;
        assert_eq!(
            install_failed_reason(log).as_deref(),
            Some("Package is not signed")
        );
    }

    #[test]
    fn silence_is_treated_as_failure() {
        assert!(install_failed_reason("").is_some());
    }

    #[test]
    fn install_targets_the_dev_channel() {
        let cmd = install_command("/media/developer/temp/fluxa.ipk");
        assert!(cmd.contains("luna://com.webos.appInstallService/dev/install"));
        assert!(cmd.contains("/media/developer/temp/fluxa.ipk"));
    }

    #[test]
    fn device_names_arrive_percent_encoded() {
        assert_eq!(percent_decode("Living%20Room%20TV"), "Living Room TV");
        assert_eq!(percent_decode("Plain"), "Plain");
    }
}
