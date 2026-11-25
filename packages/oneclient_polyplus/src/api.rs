use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;
use uuid::Uuid;

use oneclient_auth::MinecraftAccount;

use crate::session::{build_socket_client, login};
use crate::{PlusError, base_url};

pub(crate) const TOKEN_LIFETIME: Duration = Duration::from_secs(100 * 60);

static CLIENT: LazyLock<Result<reqwest::Client, String>> =
    LazyLock::new(|| build_socket_client().map_err(|err| err.to_string()));

static TOKENS: LazyLock<Mutex<HashMap<(String, Uuid), (String, Instant)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub(crate) fn client() -> Result<&'static reqwest::Client, PlusError> {
    CLIENT
        .as_ref()
        .map_err(|err| PlusError::Client(err.clone()))
}

/// Returns a cached backend token or logs in again. `stale` makes the caller
/// reject the cached token after a 401 so the next call fetches a fresh one.
pub(crate) async fn token_for(
    account: &MinecraftAccount,
    stale: Option<&str>,
) -> Result<String, PlusError> {
    let key = (base_url(), account.id);
    let mut tokens = TOKENS.lock().await;
    if let Some((token, issued)) = tokens.get(&key)
        && issued.elapsed() < TOKEN_LIFETIME
        && stale != Some(token.as_str())
    {
        return Ok(token.clone());
    }

    let token = login(client()?, account).await?;
    tokens.insert(key, (token.clone(), Instant::now()));
    Ok(token)
}
