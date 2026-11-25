//! A thin client for the instance-sharing backend.
//!
//! The launcher uploads shared instances here and verifies share codes
//! against the same host. Authentication proves the Microsoft session to
//! Mojang itself, so no credentials ever touch the backend.

pub mod api;
pub mod instances;
mod session;

use std::time::Duration;

use oneclient_common::constants;
use oneclient_net::RequestError;

pub(crate) const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

static BASE_URL: std::sync::RwLock<Option<String>> = std::sync::RwLock::new(None);

#[derive(Debug, thiserror::Error)]
pub enum PlusError {
    #[error(transparent)]
    Auth(#[from] oneclient_auth::AuthError),

    #[error("no Microsoft account is signed in")]
    NoAccount,

    #[error("Mojang rejected the session join (HTTP {status})")]
    JoinRejected { status: u16 },

    #[error("the backend rejected the login for {username} (HTTP {status})")]
    LoginRejected { username: String, status: u16 },

    #[error("sharing is off because the Terms of Service and Privacy Policy were declined")]
    ConsentRequired,

    #[error("sharing needs a Microsoft account")]
    OfflineAccount,

    #[error("backend request failed (HTTP {status}): {body}")]
    Api { status: u16, body: String },

    #[error("could not build the client: {0}")]
    Client(String),

    #[error(transparent)]
    Request(#[from] RequestError),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Url(#[from] url::ParseError),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl PlusError {
    #[must_use]
    pub fn status(&self) -> Option<u16> {
        match self {
            Self::Api { status, .. } | Self::Request(RequestError::HttpStatus { status, .. }) => {
                Some(*status)
            }
            _ => None,
        }
    }

    #[must_use]
    pub fn is_not_found(&self) -> bool {
        self.status() == Some(404)
    }

    #[must_use]
    pub fn is_forbidden(&self) -> bool {
        self.status() == Some(403)
    }

    #[must_use]
    pub fn is_unauthorized(&self) -> bool {
        self.status() == Some(401)
    }
}

pub fn plus_blocked() -> bool {
    oneclient_common::consent::blocks_url(&base_url())
}

pub fn set_base_url(url: &str) {
    if let Ok(mut current) = BASE_URL.write() {
        *current = Some(url.to_string());
    }
}

pub fn base_url() -> String {
    let local = cfg!(debug_assertions)
        .then(|| std::env::var("ONECLIENT_PLUS_URL").ok())
        .flatten()
        .filter(|url| !url.trim().is_empty());
    local
        .or_else(|| BASE_URL.read().ok().and_then(|url| url.clone()))
        .as_deref()
        .unwrap_or(constants::PLUS_BACKEND_URL)
        .trim_end_matches('/')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_api_failure_reports_its_status() {
        let err = PlusError::Api {
            status: 404,
            body: String::new(),
        };
        assert!(err.is_not_found());
    }

    #[test]
    fn a_rejected_login_says_nothing_about_the_resource() {
        assert!(!PlusError::JoinRejected { status: 403 }.is_forbidden());
        let login = PlusError::LoginRejected {
            username: String::new(),
            status: 404,
        };
        assert!(!login.is_not_found());
    }

    #[test]
    fn base_url_follows_the_pushed_setting() {
        set_base_url("https://plus-staging.example.test/");
        assert_eq!(base_url(), "https://plus-staging.example.test");
        set_base_url(constants::PLUS_BACKEND_URL);
        assert_eq!(base_url(), constants::PLUS_BACKEND_URL);
    }
}
