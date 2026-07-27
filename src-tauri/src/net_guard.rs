use std::net::IpAddr;

fn is_blocked_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
                || (v4.octets()[0] == 100 && (64..=127).contains(&v4.octets()[1]))
            // CGNAT 100.64.0.0/10
        }
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_blocked_ip(&IpAddr::V4(v4));
            }
            v6.is_loopback()
                || v6.is_unspecified()
                || (v6.segments()[0] & 0xfe00) == 0xfc00 // unique local fc00::/7
                || (v6.segments()[0] & 0xffc0) == 0xfe80 // link-local fe80::/10
        }
    }
}

/// Resolves `url`'s host and rejects it if any resolved address is loopback/private/
/// link-local -- addon-supplied URLs are otherwise free to point at internal services.
async fn resolve_public_host(url_str: &str) -> Result<(String, Vec<std::net::SocketAddr>), String> {
    let url = reqwest::Url::parse(url_str).map_err(|_| "invalid url".to_string())?;
    match url.scheme() {
        "http" | "https" => {}
        other => return Err(format!("unsupported scheme: {other}")),
    }
    let host = url
        .host_str()
        .ok_or_else(|| "url has no host".to_string())?;
    let port = url.port_or_known_default().unwrap_or(80);
    let addrs: Vec<std::net::SocketAddr> = tokio::net::lookup_host((host, port))
        .await
        .map_err(|e| format!("dns lookup failed: {e}"))?
        .collect();
    if addrs.is_empty() {
        return Err("dns lookup returned no addresses".to_string());
    }
    for addr in &addrs {
        if is_blocked_ip(&addr.ip()) {
            return Err("refusing to fetch a local/private address".to_string());
        }
    }
    Ok((host.to_string(), addrs))
}

fn block_private_redirects() -> reqwest::redirect::Policy {
    reqwest::redirect::Policy::custom(|attempt| {
        let blocked = attempt
            .url()
            .host_str()
            .and_then(|h| {
                h.trim_start_matches('[')
                    .trim_end_matches(']')
                    .parse::<IpAddr>()
                    .ok()
            })
            .is_some_and(|ip| is_blocked_ip(&ip));
        if blocked {
            attempt.error("refusing to redirect to a local/private address")
        } else if attempt.previous().len() > 10 {
            attempt.error("too many redirects")
        } else {
            attempt.follow()
        }
    })
}

/// Builds a client whose connection to `url`'s host is pinned to the addresses
/// vetted here, so a rebinding DNS entry can't swap in a private IP between the
/// check and the request.
pub async fn vetted_client(
    url_str: &str,
    timeout: std::time::Duration,
) -> Result<reqwest::Client, String> {
    let (host, addrs) = resolve_public_host(url_str).await?;
    reqwest::Client::builder()
        .timeout(timeout)
        .redirect(block_private_redirects())
        .resolve_to_addrs(&host, &addrs)
        .build()
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipv4_mapped_ipv6_does_not_escape_blocklist() {
        for s in [
            "::ffff:127.0.0.1",
            "::ffff:169.254.169.254",
            "::ffff:10.0.0.1",
        ] {
            let ip: IpAddr = s.parse().unwrap();
            assert!(is_blocked_ip(&ip), "{s} should be blocked");
        }
    }
}
