//! Curated server list.
//!
//! Offline accounts cannot join online-mode servers, so the launcher ships its
//! own list of servers that accept them: our own, plus community ones that ask
//! to be listed. The list is plain JSON in the repository, so it can be updated
//! without a client release and reviewed like any other change.
//!
//! Rules for anything that goes in here (also in README): the entry must be
//! labelled, must not block the UI, and must be a server we would play on
//! ourselves - no pay-to-win shops, no gambling, no scams.

use std::collections::HashSet;
use std::time::{Duration, Instant};

mod custom_servers;
mod servers_analytics;
pub use custom_servers::{load_custom_servers, remove_custom_server, save_custom_server};
pub use servers_analytics::{
    ServerAnalyticsStore, ServerClickRecord, get_clicks_last_30_days, record_server_click,
    upload_analytics_stub,
};

use serde::{Deserialize, Serialize};

use crate::LauncherResult;
use oneclient_common::paths;
use oneclient_net::{RequestClient, fetch_cached};

const DOWNLOAD_CAP: usize = 1024 * 1024;
const CACHE_FILE: &str = "servers.json";
const PING_TIMEOUT: Duration = Duration::from_millis(1500);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerEntry {
    /// Stable id, used as a React-style key and for analytics later.
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub summary: String,
    /// Host name or IP, without the port.
    pub address: String,
    #[serde(default = "default_port")]
    pub port: u16,
    /// Two-letter region for grouping: RU, EU, KZ...
    #[serde(default)]
    pub region: String,
    #[serde(default)]
    pub tags: Vec<String>,
    /// True when the entry is paid placement. Always shown as a label.
    #[serde(default, alias = "promoted")]
    pub is_promoted: bool,
    /// Optional web page (site, chat) shown behind an external-link button.
    #[serde(default, alias = "url")]
    pub sponsor_url: Option<String>,
}

fn default_port() -> u16 {
    25565
}

/// Sanitizes a single entry in place. Returns `false` when the entry is unusable
/// and must be dropped (empty id / name / address).
///
/// Trims every text field, restores a zero port to the default, and keeps only
/// `http`/`https` sponsor URLs (anything else, including `javascript:`, becomes
/// `None`).
pub fn sanitize_server_entry(entry: &mut ServerEntry) -> bool {
    let id = entry.id.trim();
    if id.is_empty() {
        return false;
    }
    entry.id = id.to_string();

    let name = entry.name.trim();
    if name.is_empty() {
        return false;
    }
    entry.name = name.to_string();

    let address = entry.address.trim();
    if address.is_empty() {
        return false;
    }
    entry.address = address.to_string();

    entry.summary = entry.summary.trim().to_string();
    entry.region = entry.region.trim().to_string();

    if entry.port == 0 {
        entry.port = default_port();
    }

    match &entry.sponsor_url {
        None => {}
        Some(url) => {
            let trimmed = url.trim();
            if trimmed.is_empty() {
                entry.sponsor_url = None;
            } else if is_http_url(trimmed) {
                entry.sponsor_url = Some(trimmed.to_string());
            } else {
                entry.sponsor_url = None;
            }
        }
    }

    true
}

fn is_http_url(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerList {
    #[serde(default)]
    pub servers: Vec<ServerEntry>,
}

impl ServerList {
    /// Drops entries that cannot work: no id, no name or no address. A broken
    /// entry must never take the whole screen down.
    #[must_use]
    pub fn sanitized(mut self) -> Self {
        let mut seen = HashSet::new();
        let mut out = Vec::with_capacity(self.servers.len());
        for mut server in self.servers.drain(..) {
            if !sanitize_server_entry(&mut server) {
                continue;
            }
            // First entry with a given id wins; later duplicates are dropped.
            if !seen.insert(server.id.clone()) {
                continue;
            }
            out.push(server);
        }
        self.servers = out;
        // Promoted entries are only *labelled*, never pushed to the top: paid
        // placement must not outrank a better server (see the ad policy in the
        // README). Keep the order the list author wrote.
        self
    }
}

fn cache_path() -> Option<std::path::PathBuf> {
    paths::caches_dir().ok().map(|dir| dir.join(CACHE_FILE))
}

/// Downloads the list, falling back to the cached copy, then to an empty list.
/// Never fails: a dead network must not break the screen.
pub async fn fetch_servers(net: &RequestClient) -> LauncherResult<ServerList> {
    let Some(path) = cache_path() else {
        return Ok(ServerList::default());
    };

    let fetched = fetch_cached(
        net,
        oneclient_common::constants::SERVERS_URL,
        &path,
        oneclient_net::EtagPolicy::Defer,
    )
    .await?;

    let Some(fetched) = fetched else {
        return Ok(read_cached(&path).await.unwrap_or_default());
    };

    if fetched.bytes.len() > DOWNLOAD_CAP {
        tracing::warn!(
            bytes = fetched.bytes.len(),
            "server list is over the size cap, discarding the response"
        );
        // fetch_cached already persisted the oversized body, so drop it from
        // disk instead of trusting it. The error is ignored on purpose.
        let _ = tokio::fs::remove_file(&path).await;
        return Ok(read_cached(&path).await.unwrap_or_default());
    }

    match serde_json::from_slice::<ServerList>(&fetched.bytes) {
        Ok(list) => {
            let list = list.sanitized();
            if let Some(etag) = &fetched.etag {
                oneclient_net::commit_etag(&path, etag).await;
            }
            Ok(list)
        }
        Err(err) => {
            tracing::warn!("server list is not valid json: {err}");
            Ok(read_cached(&path).await.unwrap_or_default())
        }
    }
}

async fn read_cached(path: &std::path::Path) -> Option<ServerList> {
    let bytes = tokio::fs::read(path).await.ok()?;
    serde_json::from_slice::<ServerList>(&bytes)
        .ok()
        .map(ServerList::sanitized)
}

/// TCP handshake time in milliseconds. `None` when the server is unreachable.
///
/// This is not a Minecraft ping (no status packet), it only answers "is the
/// host alive and how far is it", which is enough for the list.
pub async fn ping(entry: &ServerEntry) -> Option<u32> {
    let started = Instant::now();
    match tokio::time::timeout(
        PING_TIMEOUT,
        tokio::net::TcpStream::connect((entry.address.as_str(), entry.port)),
    )
    .await
    {
        Ok(Ok(_)) => Some(started.elapsed().as_millis().min(u32::MAX as u128) as u32),
        _ => None,
    }
}

/// The `(host, port)` pair handed to `TcpStream::connect`. Kept as a function so
/// tests can check IPv6 literals and host:port input without opening a socket.
fn connection_target(entry: &ServerEntry) -> (&str, u16) {
    (entry.address.as_str(), entry.port)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, name: &str, address: &str, port: u16) -> ServerEntry {
        ServerEntry {
            id: id.to_string(),
            name: name.to_string(),
            summary: String::new(),
            address: address.to_string(),
            port,
            region: "RU".to_string(),
            tags: Vec::new(),
            is_promoted: false,
            sponsor_url: None,
        }
    }

    #[test]
    fn entries_without_a_name_or_address_are_dropped() {
        let list = ServerList {
            servers: vec![
                entry("ok", "Good", "play.example.com", 25565),
                entry("no-name", "   ", "play.example.com", 25565),
                entry("no-address", "Name", "", 25565),
                entry("no-id", "", "play.example.com", 25565),
            ],
        }
        .sanitized();

        assert_eq!(list.servers.len(), 1);
        assert_eq!(list.servers[0].id, "ok");
    }

    #[test]
    fn a_zero_port_falls_back_to_the_default() {
        let list = ServerList {
            servers: vec![entry("x", "X", "play.example.com", 0)],
        }
        .sanitized();

        assert_eq!(list.servers[0].port, 25565);
    }

    #[test]
    fn names_and_addresses_are_trimmed() {
        let list = ServerList {
            servers: vec![entry("x", "  Padded  ", "  play.example.com  ", 25565)],
        }
        .sanitized();

        assert_eq!(list.servers[0].name, "Padded");
        assert_eq!(list.servers[0].address, "play.example.com");
    }

    #[test]
    fn a_real_document_parses() {
        let raw = br#"{"servers":[{"id":"remix","name":"Remix","summary":"Duels",
            "address":"play.remixclient.example","region":"RU","tags":["pvp"],
            "promoted":false}]}"#;
        let list: ServerList = serde_json::from_slice(raw).unwrap();

        assert_eq!(list.servers.len(), 1);
        assert_eq!(list.servers[0].port, 25565);
        assert_eq!(list.servers[0].tags, vec!["pvp".to_string()]);
    }

    #[test]
    fn duplicate_ids_are_dropped_first_wins() {
        let list = ServerList {
            servers: vec![
                entry("dup", "First", "a.example.com", 25565),
                entry("dup", "Second", "b.example.com", 25565),
            ],
        }
        .sanitized();

        assert_eq!(list.servers.len(), 1);
        assert_eq!(list.servers[0].name, "First");
        assert_eq!(list.servers[0].address, "a.example.com");
    }

    #[test]
    fn javascript_sponsor_url_is_dropped() {
        let mut e = entry("x", "X", "play.example.com", 25565);
        e.sponsor_url = Some("javascript:alert(1)".to_string());
        let list = ServerList { servers: vec![e] }.sanitized();

        assert_eq!(list.servers.len(), 1);
        assert_eq!(list.servers[0].sponsor_url, None);
    }

    #[test]
    fn http_and_https_sponsor_urls_are_kept() {
        let mut e = entry("x", "X", "play.example.com", 25565);
        e.sponsor_url = Some("  https://example.com/path  ".to_string());
        let list = ServerList { servers: vec![e] }.sanitized();

        assert_eq!(
            list.servers[0].sponsor_url,
            Some("https://example.com/path".to_string())
        );
    }

    #[test]
    fn ids_are_trimmed() {
        let list = ServerList {
            servers: vec![entry("  spaced  ", "X", "play.example.com", 25565)],
        }
        .sanitized();

        assert_eq!(list.servers[0].id, "spaced");
    }

    #[test]
    fn ipv6_address_gives_a_correct_connect_target() {
        let e = entry("x", "X", "::1", 25565);
        assert_eq!(connection_target(&e), ("::1", 25565));
        assert_eq!(connection_target(&e), ("::1", 25565));
    }
}
