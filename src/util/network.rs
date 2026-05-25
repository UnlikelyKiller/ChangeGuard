use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

/// Checks if a port is open and reachable at the given host and port.
pub fn is_host_port_reachable(host: &str, port: u16, timeout: Duration) -> bool {
    if let Ok(addrs) = (host, port).to_socket_addrs() {
        for addr in addrs {
            if TcpStream::connect_timeout(&addr, timeout).is_ok() {
                return true;
            }
        }
    }
    false
}

/// Helper to parse a base URL (e.g. "http://127.0.0.1:8081" or "http://localhost")
/// and check if it is reachable.
pub fn is_url_reachable(url: &str, timeout: Duration) -> bool {
    let stripped = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .unwrap_or(url);

    let host_port = stripped.split('/').next().unwrap_or(stripped);

    let parts: Vec<&str> = host_port.split(':').collect();
    let host = parts[0];
    let port = if parts.len() > 1 {
        parts[1].parse::<u16>().unwrap_or(80)
    } else if url.starts_with("https://") {
        443
    } else {
        80
    };

    is_host_port_reachable(host, port, timeout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_url_reachable_invalid() {
        // Unused/invalid port should return false quickly
        assert!(!is_url_reachable(
            "http://127.0.0.1:65534",
            Duration::from_millis(50)
        ));
    }
}
