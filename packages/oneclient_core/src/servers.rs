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

use std::time::{Duration, Instant};

mod servers_analytics;
mod custom_servers;
pub use servers_analytics::{record_server_click, get_clicks_last_30_days, upload_analytics_stub, ServerClickRecord, ServerAnalyticsStore};
pub use custom_servers::{load_custom_servers, save_custom_server, remove_custom_server};

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
        self.servers.retain(|server| {
            !server.id.trim().is_empty()
                && !server.name.trim().is_empty()
                && !server.address.trim().is_empty()
        });
        for server in &mut self.servers {
            server.name = server.name.trim().to_string();
            server.address = server.address.trim().to_string();
            if server.port == 0 {
                server.port = default_port();
            }
        }
        // Sort so promoted servers are always at the top
        self.servers.sort_by(|a, b| b.is_promoted.cmp(&a.is_promoted));
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
            "server list is over the size cap, keeping the cached one"
        );
        return Ok(read_cached(&path).await.unwrap_or_default());
    }

    match serde_json::from_slice::<ServerList>(&fetched.bytes) {
        Ok(list) => Ok(list.sanitized()),
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
    let target = format!("{}:{}", entry.address, entry.port);
    let started = Instant::now();
    match tokio::time::timeout(PING_TIMEOUT, tokio::net::TcpStream::connect(&target)).await {
        Ok(Ok(_)) => Some(started.elapsed().as_millis().min(u32::MAX as u128) as u32),
        _ => None,
    }
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
}
