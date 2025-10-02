use http::HeaderValue;
use std::net::{IpAddr, Ipv4Addr};
use url::Url;
use tracing::info;

/// Check if an IPv4 address is in a private range
fn is_private_ipv4(ip: &Ipv4Addr) -> bool {
    let [a, b, _, _] = ip.octets();
    (a == 10) || (a == 172 && (16..=31).contains(&b)) || (a == 192 && b == 168)
}

/// Detect local non-loopback IPv4 addresses at startup
pub fn detect_local_ips() -> Vec<Ipv4Addr> {
    use std::net::UdpSocket;

    let mut ips = Vec::new();

    // Technique: Connect UDP socket to a public IP (doesn't actually send)
    // to determine which local interface would be used
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if let Ok(()) = socket.connect("8.8.8.8:80") {
            if let Ok(addr) = socket.local_addr() {
                if let IpAddr::V4(v4) = addr.ip() {
                    if !v4.is_loopback() {
                        ips.push(v4);
                        info!("Detected local IP for CORS: {}", v4);
                    }
                }
            }
        }
    }

    ips
}

/// Determine if an origin is allowed based on CORS policy
///
/// This function checks the browser's Origin header against:
/// 1. Exact match with PUBLIC_URL (if configured)
/// 2. Exact match with detected local IPs (allows same-machine access)
/// 3. Localhost/loopback addresses (always allowed for dev)
/// 4. Private IPv4 ranges (if `allow_private` is true)
pub fn origin_allowed(
    origin: &HeaderValue,
    public_url: Option<&str>,
    local_ips: &[Ipv4Addr],
    allow_private: bool,
) -> bool {
    // Parse the origin header
    let origin_str = match origin.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    let url = match Url::parse(origin_str) {
        Ok(u) => u,
        Err(_) => return false,
    };

    let host = match url.host_str() {
        Some(h) => h,
        None => return false,
    };

    // 1) Exact match with PUBLIC_URL
    if let Some(pub_url) = public_url {
        if let Ok(p) = Url::parse(pub_url) {
            if p.scheme() == url.scheme()
                && p.host_str() == url.host_str()
                && p.port_or_known_default() == url.port_or_known_default()
            {
                return true;
            }
        }
    }

    // 2) Check if origin matches any detected local IP (any port allowed)
    if let Ok(IpAddr::V4(origin_ip)) = host.parse::<IpAddr>() {
        if local_ips.contains(&origin_ip) {
            return true;
        }
    }

    // 3) Localhost / loopback (always allowed for development)
    if host.eq_ignore_ascii_case("localhost")
        || host == "[::1]"
        || host == "::1"
        || host.starts_with("127.")
    {
        return true;
    }

    // 4) Optional: All private IPv4 ranges (LAN-wide convenience)
    if allow_private {
        if let Ok(IpAddr::V4(v4)) = host.parse::<IpAddr>() {
            if is_private_ipv4(&v4) {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderValue;

    #[test]
    fn test_localhost_allowed() {
        let local_ips = vec![];

        let origin = HeaderValue::from_static("http://localhost:3000");
        assert!(origin_allowed(&origin, None, &local_ips, false));

        let origin = HeaderValue::from_static("http://127.0.0.1:5173");
        assert!(origin_allowed(&origin, None, &local_ips, false));

        let origin = HeaderValue::from_static("http://[::1]:8080");
        assert!(origin_allowed(&origin, None, &local_ips, false));
    }

    #[test]
    fn test_public_url_exact_match() {
        let local_ips = vec![];

        let origin = HeaderValue::from_static("https://app.example.com");
        assert!(origin_allowed(
            &origin,
            Some("https://app.example.com"),
            &local_ips,
            false
        ));

        // Different port should not match
        let origin = HeaderValue::from_static("https://app.example.com:8080");
        assert!(!origin_allowed(
            &origin,
            Some("https://app.example.com"),
            &local_ips,
            false
        ));

        // Different scheme should not match
        let origin = HeaderValue::from_static("http://app.example.com");
        assert!(!origin_allowed(
            &origin,
            Some("https://app.example.com"),
            &local_ips,
            false
        ));
    }

    #[test]
    fn test_private_ips_when_allowed() {
        let local_ips = vec![];

        // 192.168.x.x
        let origin = HeaderValue::from_static("http://192.168.1.50:5173");
        assert!(origin_allowed(&origin, None, &local_ips, true));
        assert!(!origin_allowed(&origin, None, &local_ips, false));

        // 10.x.x.x
        let origin = HeaderValue::from_static("http://10.0.0.1:3000");
        assert!(origin_allowed(&origin, None, &local_ips, true));
        assert!(!origin_allowed(&origin, None, &local_ips, false));

        // 172.16-31.x.x
        let origin = HeaderValue::from_static("http://172.20.0.1:8080");
        assert!(origin_allowed(&origin, None, &local_ips, true));
        assert!(!origin_allowed(&origin, None, &local_ips, false));
    }

    #[test]
    fn test_public_ip_not_allowed() {
        let local_ips = vec![];

        let origin = HeaderValue::from_static("http://1.2.3.4:5173");
        assert!(!origin_allowed(&origin, None, &local_ips, false));
        assert!(!origin_allowed(&origin, None, &local_ips, true));
    }

    #[test]
    fn test_invalid_origin_rejected() {
        let local_ips = vec![];

        let origin = HeaderValue::from_static("not-a-url");
        assert!(!origin_allowed(&origin, None, &local_ips, false));
    }

    #[test]
    fn test_detected_local_ip_allowed() {
        // Simulate detected local IP
        let local_ips = vec![Ipv4Addr::new(192, 168, 1, 100)];

        // Origin matching detected IP should be allowed (any port)
        let origin = HeaderValue::from_static("http://192.168.1.100:5173");
        assert!(origin_allowed(&origin, None, &local_ips, false));

        let origin = HeaderValue::from_static("http://192.168.1.100:3000");
        assert!(origin_allowed(&origin, None, &local_ips, false));

        // Different IP should not match (unless allow_private is true)
        let origin = HeaderValue::from_static("http://192.168.1.200:5173");
        assert!(!origin_allowed(&origin, None, &local_ips, false));
        assert!(origin_allowed(&origin, None, &local_ips, true)); // allowed via private range
    }
}
