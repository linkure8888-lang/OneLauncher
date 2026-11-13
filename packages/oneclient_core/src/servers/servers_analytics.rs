use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use oneclient_common::paths;

const ANALYTICS_FILE: &str = "analytics.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerClickRecord {
    pub server_id: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerAnalyticsStore {
    #[serde(default)]
    pub clicks: Vec<ServerClickRecord>,
}

fn analytics_path() -> Option<PathBuf> {
    paths::config_dir().ok().map(|dir| dir.join(ANALYTICS_FILE))
}

pub async fn record_server_click(server_id: &str) {
    let Some(path) = analytics_path() else {
        return;
    };

    let mut store = load_store(&path).await;
    store.clicks.push(ServerClickRecord {
        server_id: server_id.to_string(),
        timestamp: Utc::now().timestamp(),
    });

    if let Ok(bytes) = serde_json::to_vec(&store) {
        if let Some(parent) = path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }
        let _ = tokio::fs::write(&path, bytes).await;
    }
}

pub async fn get_clicks_last_30_days(server_id: &str) -> usize {
    let Some(path) = analytics_path() else {
        return 0;
    };
    let store = load_store(&path).await;
    let cutoff = (Utc::now() - Duration::days(30)).timestamp();

    store.clicks
        .iter()
        .filter(|c| c.server_id == server_id && c.timestamp >= cutoff)
        .count()
}

pub async fn upload_analytics_stub() -> crate::LauncherResult<()> {
    let Some(path) = analytics_path() else {
        return Ok(());
    };
    let _store = load_store(&path).await;
    tracing::info!("Analytics upload stub: local stats successfully saved and prepared for upload.");
    Ok(())
}

async fn load_store(path: &std::path::Path) -> ServerAnalyticsStore {
    if let Ok(bytes) = tokio::fs::read(path).await {
        if let Ok(store) = serde_json::from_slice::<ServerAnalyticsStore>(&bytes) {
            return store;
        }
    }
    ServerAnalyticsStore::default()
}
