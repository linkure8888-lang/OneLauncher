use crate::servers::ServerEntry;
use oneclient_common::paths;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const CUSTOM_SERVERS_FILE: &str = "servers_custom.json";

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

pub async fn load_custom_servers() -> Vec<ServerEntry> {
    let Some(path) = custom_servers_path() else {
        return Vec::new();
    };
    if let Ok(bytes) = tokio::fs::read(&path).await {
        if let Ok(store) = serde_json::from_slice::<CustomServerStore>(&bytes) {
            return store.servers;
        }
    }
    Vec::new()
}

pub async fn save_custom_server(entry: ServerEntry) -> crate::LauncherResult<()> {
    let Some(path) = custom_servers_path() else {
        return Ok(());
    };
    let mut store = CustomServerStore {
        servers: load_custom_servers().await,
    };
    if let Some(pos) = store.servers.iter().position(|s| s.id == entry.id) {
        store.servers[pos] = entry;
    } else {
        store.servers.push(entry);
    }

    let bytes = serde_json::to_vec_pretty(&store)?;
    if let Some(parent) = path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    tokio::fs::write(&path, bytes).await?;
    Ok(())
}

pub async fn remove_custom_server(id: &str) -> crate::LauncherResult<()> {
    let Some(path) = custom_servers_path() else {
        return Ok(());
    };
    let mut store = CustomServerStore {
        servers: load_custom_servers().await,
    };
    store.servers.retain(|s| s.id != id);

    let bytes = serde_json::to_vec_pretty(&store)?;
    if let Some(parent) = path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    tokio::fs::write(&path, bytes).await?;
    Ok(())
}
