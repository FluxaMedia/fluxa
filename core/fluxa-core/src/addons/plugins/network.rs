use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use url::{Host, Url};

pub fn is_public_plugin_host(host: Host<&str>) -> bool {
    match host {
        Host::Ipv4(ip) => is_public_plugin_ip(IpAddr::V4(ip)),
        Host::Ipv6(ip) => is_public_plugin_ip(IpAddr::V6(ip)),
        Host::Domain(domain) => {
            let domain = domain.trim_end_matches('.').to_ascii_lowercase();
            domain != "localhost"
                && !domain.ends_with(".localhost")
                && domain
                    .parse::<Ipv4Addr>()
                    .map(|ip| is_public_plugin_ip(IpAddr::V4(ip)))
                    .unwrap_or(true)
        }
    }
}

pub fn plugin_network_address_allowed(address: &str) -> bool {
    address.parse::<IpAddr>().is_ok_and(is_public_plugin_ip)
}

pub fn plugin_network_address_bytes_allowed(bytes: &[u8]) -> bool {
    match bytes.len() {
        4 => is_public_plugin_ip(IpAddr::V4(Ipv4Addr::from(
            <[u8; 4]>::try_from(bytes).unwrap(),
        ))),
        16 => is_public_plugin_ip(IpAddr::V6(Ipv6Addr::from(
            <[u8; 16]>::try_from(bytes).unwrap(),
        ))),
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

fn is_public_plugin_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => is_public_plugin_ipv4(ip),
        IpAddr::V6(ip) => is_public_plugin_ipv6(ip),
    }
}

fn is_public_plugin_ipv4(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();
    !ip.is_private()
        && !ip.is_loopback()
        && !ip.is_link_local()
        && !ip.is_unspecified()
        && !ip.is_broadcast()
        && !ip.is_multicast()
        && a != 0
        && a < 240
        && !(a == 100 && (64..=127).contains(&b))
        && !(a == 192 && b == 0 && c == 0)
        && !(a == 198 && (b == 18 || b == 19))
}

fn is_public_plugin_ipv6(ip: Ipv6Addr) -> bool {
    let segments = ip.segments();
    if let Some(embedded) = ip.to_ipv4_mapped() {
        return is_public_plugin_ipv4(embedded);
    }
    if segments[..6] == [0, 0, 0, 0, 0, 0] {
        return false;
    }
    if segments[..6] == [0x64, 0xff9b, 0, 0, 0, 0] {
        let [a, b] = segments[6].to_be_bytes();
        let [c, d] = segments[7].to_be_bytes();
        return is_public_plugin_ipv4(Ipv4Addr::new(a, b, c, d));
    }
    if segments[0] == 0x2002 {
        let [a, b] = segments[1].to_be_bytes();
        let [c, d] = segments[2].to_be_bytes();
        return is_public_plugin_ipv4(Ipv4Addr::new(a, b, c, d));
    }
    !ip.is_loopback()
        && !ip.is_unspecified()
        && !ip.is_multicast()
        && !ip.is_unique_local()
        && !ip.is_unicast_link_local()
        && segments[0] & 0xffc0 != 0xfec0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_that_reach_internal_hosts_are_rejected() {
        for url in [
            "http://127.0.0.1/",
            "http://127.1/",
            "http://2130706433/",
            "http://0x7f.0.0.1/",
            "http://0177.0.0.1/",
            "http://example.com@127.0.0.1/",
            "http://example.com:80@169.254.169.254/latest/meta-data/",
            "http://[::1]/",
            "http://[::ffff:127.0.0.1]/",
            "http://[::ffff:7f00:1]/",
            "http://[::ffff:169.254.169.254]/",
            "http://[::127.0.0.1]/",
            "http://[64:ff9b::7f00:1]/",
            "http://[2002:7f00:1::]/",
            "http://[fec0::1]/",
            "http://[ff02::1]/",
            "http://0.0.0.0/",
            "http://0.1.2.3/",
            "http://10.0.0.1/",
            "http://192.168.1.1/",
            "http://172.16.0.1/",
            "http://100.64.0.1/",
            "http://192.0.0.1/",
            "http://198.18.0.1/",
            "http://224.0.0.1/",
            "http://240.0.0.1/",
            "http://localhost./",
            "http://foo.localhost/",
            "http://LOCALHOST/",
        ] {
            assert!(!plugin_url_allowed(url), "{url} must be rejected");
        }
    }

    #[test]
    fn public_urls_are_allowed() {
        for url in [
            "https://example.com/",
            "http://8.8.8.8/",
            "http://[2606:4700:4700::1111]/",
            "http://example.com@example.org/",
        ] {
            assert!(plugin_url_allowed(url), "{url} must be allowed");
        }
    }

    #[test]
    fn resolved_addresses_are_classified_like_urls() {
        for address in [
            "::ffff:10.0.0.1",
            "::ffff:7f00:1",
            "64:ff9b::a00:1",
            "2002:c0a8:101::",
            "ff00::1",
            "0.0.0.1",
            "255.255.255.255",
            "198.19.255.255",
        ] {
            assert!(!plugin_network_address_allowed(address), "{address}");
        }
        assert!(plugin_network_address_allowed("93.184.216.34"));
    }
}
