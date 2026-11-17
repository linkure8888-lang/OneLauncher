//! Announcing our identity to a self-hosted RemixServer.
//!
//! Offline mode trusts the nickname a client sends, so a hub can only tell
//! players apart if someone proves the name is theirs. The launcher owns a
//! secret; it hands that secret to the hub's auth gate right before launching,
//! and the gate binds the nickname to it on first join.
//!
//! Only loopback servers ever see the secret: a public deployment would need TLS
//! and an explicit opt-in, and silently posting a credential to any server the
//! player clicks would be worse than a nickname being stolen.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::time::Duration;

use oneclient_common::paths;

/// The gate's loopback port; it is deliberately not configurable from the
/// catalogue, so a hostile server entry cannot redirect the secret elsewhere.
const AUTH_PORT: u16 = 8765;

const TOKEN_FILE: &str = "auth.key";
const TOKEN_BYTES: usize = 32;
const CONNECT_TIMEOUT: Duration = Duration::from_millis(400);

fn token_path() -> Option<PathBuf> {
    paths::data_dir().ok().map(|dir| dir.join(TOKEN_FILE))
}

/// Reads the install's secret, creating it on first use.
///
/// The same file must survive restarts: a new secret would look like a thief to
/// the hub and lock the player out of their own nickname.
pub fn token() -> Option<String> {
    let path = token_path()?;

    if let Ok(existing) = std::fs::read_to_string(&path) {
        let trimmed = existing.trim();
        if trimmed.len() >= TOKEN_BYTES * 2 {
            return Some(trimmed.to_string());
        }
    }

    let generated = random_hex(TOKEN_BYTES);

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok()?;
    }

    std::fs::write(&path, &generated).ok()?;
    restrict_permissions(&path);

    Some(generated)
}

fn random_hex(bytes: usize) -> String {
    let mut buffer = vec![0u8; bytes];
    for chunk in buffer.chunks_mut(8) {
        let value: u64 = rand::random();
        chunk.copy_from_slice(&value.to_le_bytes()[..chunk.len()]);
    }
    buffer.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(unix)]
fn restrict_permissions(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &std::path::Path) {}

/// Splits `host:port`, tolerating IPv6 in brackets.
pub fn split_address(address: &str) -> Option<(String, u16)> {
    let address = address.trim();
    if let Some(rest) = address.strip_prefix('[') {
        let (host, tail) = rest.split_once(']')?;
        let port = tail.strip_prefix(':')?.parse().ok()?;
        return Some((host.to_string(), port));
    }

    let (host, port) = address.rsplit_once(':')?;
    if host.is_empty() {
        return None;
    }
    Some((host.to_string(), port.parse().ok()?))
}

pub fn is_loopback_host(host: &str) -> bool {
    let host = host.trim().trim_matches(|ch| ch == '[' || ch == ']');
    matches!(host, "localhost" | "127.0.0.1" | "::1")
        || host
            .parse::<std::net::IpAddr>()
            .is_ok_and(|ip| ip.is_loopback())
}

fn percent_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

fn post_json(addr: SocketAddr, path: &str, body: &str) -> Option<String> {
    let mut stream = TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT).ok()?;
    stream.set_write_timeout(Some(CONNECT_TIMEOUT)).ok()?;
    stream.set_read_timeout(Some(CONNECT_TIMEOUT)).ok()?;

    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n{body}",
        len = body.len()
    );
    stream.write_all(request.as_bytes()).ok()?;

    let mut response = String::new();
    stream.read_to_string(&mut response).ok()?;

    // The status line is all we need: the gate answers 200 for an accepted
    // session and 403 for a nickname someone else already claimed.
    Some(response)
}

/// Tells the hub at `address` who is about to join.
///
/// Returns false when the server has no gate, is unreachable, or refused us —
/// the caller logs it and launches anyway, because a server without auth must
/// keep working.
pub fn announce(address: &str, nickname: &str) -> bool {
    let Some((host, _port)) = split_address(address) else {
        return false;
    };
    if !is_loopback_host(&host) {
        return false;
    }
    let Some(token) = token() else {
        return false;
    };
    let Ok(addr) = format!("127.0.0.1:{AUTH_PORT}").parse() else {
        return false;
    };

    announce_to(addr, nickname, &token)
}

fn announce_to(addr: SocketAddr, nickname: &str, token: &str) -> bool {
    let body = format!(
        "nick={}&token={}",
        percent_encode(nickname),
        percent_encode(token)
    );

    post_json(addr, "/session", &body).is_some_and(|response| response.starts_with("HTTP/1.1 200"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn split_address_handles_ipv4_and_ipv6() {
        assert_eq!(
            split_address("127.0.0.1:25565"),
            Some(("127.0.0.1".to_string(), 25565))
        );
        assert_eq!(
            split_address("[::1]:25565"),
            Some(("::1".to_string(), 25565))
        );
        assert_eq!(split_address("nohost"), None);
        assert_eq!(split_address(":25565"), None);
    }

    #[test]
    fn only_loopback_is_trusted_with_the_secret() {
        assert!(is_loopback_host("127.0.0.1"));
        assert!(is_loopback_host("localhost"));
        assert!(is_loopback_host("[::1]"));
        assert!(is_loopback_host("127.0.0.5"));
        assert!(!is_loopback_host("play.example.com"));
        assert!(!is_loopback_host("192.168.0.10"));
    }

    #[test]
    fn nickname_is_form_encoded() {
        assert_eq!(percent_encode("Steve"), "Steve");
        assert_eq!(percent_encode("a b&c=d"), "a%20b%26c%3Dd");
    }

    #[test]
    fn the_gate_sees_a_well_formed_session_request() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let gate = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            let mut buffer = [0u8; 2048];
            let read = socket.read(&mut buffer).unwrap();
            let request = String::from_utf8_lossy(&buffer[..read]).to_string();
            socket
                .write_all(
                    b"HTTP/1.1 200 OK
Content-Length: 2

ok",
                )
                .unwrap();
            request
        });

        let response = post_json(addr, "/session", "nick=Steve&token=abc").unwrap();
        let request = gate.join().unwrap();

        assert!(request.starts_with("POST /session HTTP/1.1"));
        assert!(request.contains("Content-Length: 20"));
        assert!(request.ends_with("nick=Steve&token=abc"));
        assert!(response.starts_with("HTTP/1.1 200"));
    }

    fn gate_answering(status: &'static str) -> (SocketAddr, std::thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            let mut buffer = [0u8; 2048];
            let read = socket.read(&mut buffer).unwrap();
            let request = String::from_utf8_lossy(&buffer[..read]).to_string();
            socket
                .write_all(
                    format!(
                        "HTTP/1.1 {status}
Content-Length: 0

"
                    )
                    .as_bytes(),
                )
                .unwrap();
            request
        });
        (addr, handle)
    }

    #[test]
    fn an_accepted_session_returns_true() {
        let (addr, gate) = gate_answering("200 OK");
        assert!(announce_to(addr, "Steve", "a-very-secret-token"));

        let request = gate.join().unwrap();
        assert!(request.contains("nick=Steve&token=a-very-secret-token"));
    }

    #[test]
    fn a_refused_session_returns_false() {
        let (addr, gate) = gate_answering("403 Forbidden");
        assert!(!announce_to(addr, "Steve", "someone-elses-token"));
        gate.join().unwrap();
    }

    #[test]
    fn announce_ignores_remote_servers() {
        assert!(!announce("play.example.com:25565", "Steve"));
    }
}
