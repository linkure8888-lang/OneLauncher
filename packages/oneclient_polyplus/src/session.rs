use std::time::Duration;

use rand::Rng;

use oneclient_auth::MinecraftAccount;

use crate::{PlusError, REQUEST_TIMEOUT, base_url, plus_blocked};

const MOJANG_JOIN_URL: &str = "https://sessionserver.mojang.com/session/minecraft/join";

#[derive(serde::Deserialize)]
struct LoginResponse {
    token: String,
}

/// The HTTP client for the sharing backend. A plain HTTP/1.1 client keeps the
/// connection behavior identical between the upload endpoints.
pub(crate) fn build_socket_client() -> Result<reqwest::Client, reqwest::Error> {
    let builder = reqwest::Client::builder()
        .tcp_keepalive(Some(Duration::from_secs(15)))
        .connect_timeout(Duration::from_secs(10))
        .http1_only()
        .tls_backend_rustls()
        .user_agent(format!(
            "RemixClient {} ({})",
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_HOMEPAGE")
        ));

    builder.no_hickory_dns().build()
}

/// Logs into the sharing backend by proving the Mojang session: the launcher
/// joins a generated server id through Mojang, the backend verifies the same
/// join server-side, and only then hands out a token. No password ever leaves.
pub(crate) async fn login(
    client: &reqwest::Client,
    account: &MinecraftAccount,
) -> Result<String, PlusError> {
    if plus_blocked() {
        return Err(PlusError::ConsentRequired);
    }

    let server_id = generate_server_id();

    let join = client
        .post(MOJANG_JOIN_URL)
        .timeout(REQUEST_TIMEOUT)
        .header("Accept", "application/json")
        .json(&serde_json::json!({
            "accessToken": account.access_token,
            "selectedProfile": account.id.simple().to_string(),
            "serverId": server_id,
        }))
        .send()
        .await?;

    if !join.status().is_success() {
        return Err(PlusError::JoinRejected {
            status: join.status().as_u16(),
        });
    }

    let mut url = reqwest::Url::parse(&format!("{}/account/login", base_url()))?;
    url.query_pairs_mut()
        .append_pair("username", &account.username)
        .append_pair("server_id", &server_id);

    let login = client
        .post(url)
        .timeout(REQUEST_TIMEOUT)
        .header("Accept", "application/json")
        .send()
        .await?;

    if !login.status().is_success() {
        return Err(PlusError::LoginRejected {
            username: account.username.clone(),
            status: login.status().as_u16(),
        });
    }

    Ok(login.json::<LoginResponse>().await?.token)
}

fn generate_server_id() -> String {
    let mut bytes = [0u8; 20];
    rand::rng().fill_bytes(&mut bytes);
    polyio::to_hex(&bytes)
}
