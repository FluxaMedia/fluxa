use url::{Host, Url};

pub fn is_public_plugin_host(host: Host<&str>) -> bool {
    match host {
        Host::Ipv4(ip) => is_public_plugin_ipv4(ip),
        Host::Ipv6(ip) => is_public_plugin_ipv6(ip),
        Host::Domain(domain) => {
            let domain = domain.trim_end_matches('.').to_ascii_lowercase();
            domain != "localhost"
                && !domain.ends_with(".localhost")
                && domain
                    .parse::<std::net::Ipv4Addr>()
                    .map(|ip| is_public_plugin_host(Host::Ipv4(ip)))
                    .unwrap_or(true)
        }
    }
}

pub fn plugin_network_address_allowed(address: &str) -> bool {
    match address.parse::<std::net::IpAddr>() {
        Ok(std::net::IpAddr::V4(ip)) => is_public_plugin_ipv4(ip),
        Ok(std::net::IpAddr::V6(ip)) => ip
            .to_ipv4_mapped()
            .map(is_public_plugin_ipv4)
            .unwrap_or_else(|| is_public_plugin_ipv6(ip)),
        Err(_) => false,
    }
}

pub fn plugin_network_address_bytes_allowed(bytes: &[u8]) -> bool {
    match bytes.len() {
        4 => is_public_plugin_ipv4(std::net::Ipv4Addr::from(
            <[u8; 4]>::try_from(bytes).unwrap(),
        )),
        16 => {
            let ip = std::net::Ipv6Addr::from(<[u8; 16]>::try_from(bytes).unwrap());
            ip.to_ipv4_mapped()
                .map(is_public_plugin_ipv4)
                .unwrap_or_else(|| is_public_plugin_ipv6(ip))
        }
        _ => false,
    }
}

pub fn plugin_url_allowed(url: &str) -> bool {
    let Ok(parsed) = Url::parse(url) else {
        return false;
    };
    if !matches!(parsed.scheme(), "http" | "https") {
        return false;
    }
    parsed.host().is_some_and(is_public_plugin_host)
}

fn is_public_plugin_ipv4(ip: std::net::Ipv4Addr) -> bool {
    !ip.is_private()
        && !ip.is_loopback()
        && !ip.is_link_local()
        && !ip.is_unspecified()
        && !ip.is_broadcast()
        && !(ip.octets()[0] == 100 && ip.octets()[1] >= 64 && ip.octets()[1] <= 127)
}

fn is_public_plugin_ipv6(ip: std::net::Ipv6Addr) -> bool {
    !ip.is_loopback()
        && !ip.is_unspecified()
        && !ip.is_unique_local()
        && !ip.is_unicast_link_local()
}
