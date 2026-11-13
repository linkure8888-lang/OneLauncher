use crate::servers::{ServerEntry, sanitize_server_entry};
use oneclient_common::paths;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const CUSTOM_SERVERS_FILE: &str = "servers_custom.json";
const MAX_CUSTOM_SERVERS: usize = 200;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomServerStore {
    #[serde(default)]
    pub servers: Vec<ServerEntry>,
}

fn custom_servers_path() -> Option<PathBuf> {
    paths::config_dir()
        .ok()
        .map(|dir| dir.join(CUSTOM_SERVERS_FILE))
}

fn sanitize_store(store: &mut CustomServerStore) {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::with_capacity(store.servers.len());
    for mut server in std::mem::take(&mut store.servers) {
        if !sanitize_server_entry(&mut server) {
            continue;
        }
        if !seen.insert(server.id.clone()) {
            continue;
        }
        out.push(server);
    }
    store.servers = out;
}

pub async fn load_custom_servers() -> Vec<ServerEntry> {
    let Some(path) = custom_servers_path() else {
        return Vec::new();
    };
    load_store(&path).await.servers
}

/// Reads and sanitises the store at an explicit path.
///
/// Tests call it directly, so they never touch the real user folder: the tests
/// used to share one global launcher dir and failed depending on the order.
async fn load_store(path: &Path) -> CustomServerStore {
    let bytes = match tokio::fs::read(path).await {
        Ok(bytes) => bytes,
        Err(err) => {
            if err.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!(path = %path.display(), "failed to read custom servers: {err}");
            }
            return CustomServerStore::default();
        }
    };
    match serde_json::from_slice::<CustomServerStore>(&bytes) {
        Ok(mut store) => {
            sanitize_store(&mut store);
            store
        }
        Err(err) => {
            tracing::warn!(path = %path.display(), "failed to parse custom servers: {err}");
            CustomServerStore::default()
        }
    }
}

/// Writes through a temp file: a power loss mid-write must not leave a truncated
/// list behind.
async fn write_store(path: &Path, store: &CustomServerStore) -> crate::LauncherResult<()> {
    let bytes = serde_json::to_vec_pretty(store)?;
    if let Some(parent) = path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    let tmp = path.with_extension("tmp");
    tokio::fs::write(&tmp, &bytes).await?;
    tokio::fs::rename(&tmp, path).await?;
    Ok(())
}

pub async fn save_custom_server(entry: ServerEntry) -> crate::LauncherResult<()> {
    let Some(path) = custom_servers_path() else {
        return Ok(());
    };

    let mut entry = entry;
    if !sanitize_server_entry(&mut entry) {
        return Err(crate::LauncherError::Minecraft(
            "invalid custom server entry".to_string(),
        ));
    }

    let mut store = load_store(&path).await;

    let is_update = store.servers.iter().any(|s| s.id == entry.id);
    if !is_update && store.servers.len() >= MAX_CUSTOM_SERVERS {
        return Err(crate::LauncherError::Minecraft(format!(
            "too many custom servers (limit {MAX_CUSTOM_SERVERS})"
        )));
    }

    if let Some(pos) = store.servers.iter().position(|s| s.id == entry.id) {
        store.servers[pos] = entry;
    } else {
        store.servers.push(entry);
    }
    sanitize_store(&mut store);
    write_store(&path, &store).await
}

pub async fn remove_custom_server(id: &str) -> crate::LauncherResult<()> {
    let Some(path) = custom_servers_path() else {
        return Ok(());
    };
    let mut store = load_store(&path).await;
    store.servers.retain(|s| s.id != id);
    sanitize_store(&mut store);
    write_store(&path, &store).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, name: &str, address: &str, url: Option<&str>) -> ServerEntry {
        ServerEntry {
            id: id.to_string(),
            name: name.to_string(),
            summary: String::new(),
            address: address.to_string(),
            port: 0,
            region: String::new(),
            tags: Vec::new(),
            is_promoted: false,
            sponsor_url: url.map(str::to_string),
        }
    }

    /// A private file per test: no shared global state, no lock, no ordering.
    fn temp_file() -> PathBuf {
        std::env::temp_dir()
            .join(format!("remix-custom-servers-{}", uuid::Uuid::new_v4()))
            .join(CUSTOM_SERVERS_FILE)
    }

    async fn save_to(path: &Path, entry: ServerEntry) -> crate::LauncherResult<()> {
        let mut entry = entry;
        if !sanitize_server_entry(&mut entry) {
            return Err(crate::LauncherError::Minecraft(
                "invalid custom server entry".to_string(),
            ));
        }
        let mut store = load_store(path).await;
        let is_update = store.servers.iter().any(|s| s.id == entry.id);
        if !is_update && store.servers.len() >= MAX_CUSTOM_SERVERS {
            return Err(crate::LauncherError::Minecraft(format!(
                "too many custom servers (limit {MAX_CUSTOM_SERVERS})"
            )));
        }
        if let Some(pos) = store.servers.iter().position(|s| s.id == entry.id) {
            store.servers[pos] = entry;
        } else {
            store.servers.push(entry);
        }
        sanitize_store(&mut store);
        write_store(path, &store).await
    }

    #[tokio::test]
    async fn save_and_reload_preserves_data() {
        let path = temp_file();
        save_to(
            &path,
            entry(
                "my",
                "My Server",
                "play.example.com",
                Some("https://example.com"),
            ),
        )
        .await
        .unwrap();

        let loaded = load_store(&path).await;
        assert_eq!(loaded.servers.len(), 1);
        assert_eq!(loaded.servers[0].id, "my");
        assert_eq!(loaded.servers[0].name, "My Server");
        assert_eq!(loaded.servers[0].address, "play.example.com");
        assert_eq!(loaded.servers[0].port, 25565);
        assert_eq!(
            loaded.servers[0].sponsor_url,
            Some("https://example.com".to_string())
        );

        std::fs::remove_dir_all(path.parent().unwrap()).ok();
    }

    #[tokio::test]
    async fn invalid_url_is_nulled() {
        let path = temp_file();
        save_to(
            &path,
            entry(
                "bad",
                "Bad",
                "play.example.com",
                Some("javascript:alert(1)"),
            ),
        )
        .await
        .unwrap();

        let loaded = load_store(&path).await;
        assert_eq!(loaded.servers.len(), 1);
        assert_eq!(loaded.servers[0].sponsor_url, None);

        std::fs::remove_dir_all(path.parent().unwrap()).ok();
    }

    #[tokio::test]
    async fn limit_is_enforced() {
        let path = temp_file();
        for i in 0..MAX_CUSTOM_SERVERS {
            save_to(
                &path,
                entry(&format!("s{i}"), "S", "play.example.com", None),
            )
            .await
            .unwrap();
        }

        let res = save_to(&path, entry("overflow", "X", "play.example.com", None)).await;
        assert!(
            res.is_err(),
            "the limit must reject entry {MAX_CUSTOM_SERVERS}"
        );

        std::fs::remove_dir_all(path.parent().unwrap()).ok();
    }

    #[tokio::test]
    async fn an_empty_entry_is_rejected() {
        let path = temp_file();
        let res = save_to(&path, entry("", "  ", "", None)).await;

        assert!(res.is_err());
        assert!(load_store(&path).await.servers.is_empty());
    }
}
