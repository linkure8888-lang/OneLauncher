use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SocialError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Config directory not found")]
    NoConfigDir,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Friend {
    pub nickname: String,
    pub uuid: String,
    pub last_seen: u64,
}

fn config_path() -> Result<PathBuf, SocialError> {
    let proj_dirs = ProjectDirs::from("com", "remixclient", "oneclient")
        .ok_or(SocialError::NoConfigDir)?;
    let dir = proj_dirs.config_dir();
    fs::create_dir_all(dir)?;
    Ok(dir.join("friends.json"))
}

pub fn load_friends() -> Result<Vec<Friend>, SocialError> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(path)?;
    let friends: Vec<Friend> = serde_json::from_str(&data)?;
    Ok(friends)
}

pub fn save_friends(friends: &[Friend]) -> Result<(), SocialError> {
    let path = config_path()?;
    let data = serde_json::to_string_pretty(friends)?;
    fs::write(path, data)?;
    Ok(())
}

pub fn add_friend(nickname: &str, uuid: &str) -> Result<(), SocialError> {
    let mut friends = load_friends()?;
    let now = chrono::Utc::now().timestamp() as u64;

    if let Some(friend) = friends.iter_mut().find(|f| f.uuid == uuid) {
        friend.nickname = nickname.to_string();
        friend.last_seen = now;
    } else {
        friends.push(Friend {
            nickname: nickname.to_string(),
            uuid: uuid.to_string(),
            last_seen: now,
        });
    }

    save_friends(&friends)?;
    Ok(())
}

pub fn remove_friend(uuid: &str) -> Result<(), SocialError> {
    let mut friends = load_friends()?;
    let initial_len = friends.len();
    friends.retain(|f| f.uuid != uuid);
    if friends.len() != initial_len {
        save_friends(&friends)?;
    }
    Ok(())
}

pub fn init_social_subsystem() {
    match load_friends() {
        Ok(friends) => {
            tracing::info!("Local friends loaded: {}", friends.len());
            println!("Local friends loaded: {}", friends.len());
        }
        Err(e) => {
            tracing::warn!("Failed to load local friends: {}", e);
            println!("Failed to load local friends: {}", e);
        }
    }
}
