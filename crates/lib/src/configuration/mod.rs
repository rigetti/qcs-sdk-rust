//! This module is used for loading configuration that will be used to connect either to real QPUs
//! (and supporting services) or the QVM.
//!
//! By default, all settings are loaded from files located
//! under your home directory in a `.qcs` folder. `settings.toml` will be used to load general
//! settings (e.g. which URLs to connect to) and `secrets.toml` will be used to load tokens for
//! authentication. Both "settings" and "secrets" files should contain profiles. The
//! `default_profile_name` in settings sets the profile to be used when there is no override. You
//! can set the [`PROFILE_NAME_VAR`] to select a different profile. You can also use
//! [`SECRETS_PATH_VAR`] and [`SETTINGS_PATH_VAR`] to change which files are loaded.

use std::path::PathBuf;

use futures::future::try_join;
use serde::{Deserialize, Serialize};

use qcs_api::apis::configuration as api;
use secrets::Secrets;
pub use secrets::SECRETS_PATH_VAR;
pub use settings::SETTINGS_PATH_VAR;
use settings::{AuthServer, Pyquil, Settings};

use crate::configuration::LoadError::AuthServerNotFound;

mod path;
mod secrets;
mod settings;

/// All the config data that's parsed from config sources
#[derive(Clone, Debug)]
pub struct Configuration {
    api_config: api::Configuration,
    auth_server: AuthServer,
    refresh_token: Option<String>,
    /// The URL for the quilc server.
    pub quilc_url: String,
    /// The URL for the QVM server.
    pub qvm_url: String,
}

/// Setting this environment variable will change which profile is used from the loaded config files
pub const PROFILE_NAME_VAR: &str = "QCS_PROFILE_NAME";

/// Errors raised when attempting to refresh the user's token
#[derive(Debug, thiserror::Error)]
pub enum RefreshError {
    /// Error due to no token available to refresh.
    #[error("No refresh token is in secrets")]
    NoRefreshToken,
    /// Error when trying to fetch the new token.
    #[error("Error fetching new token")]
    FetchError(#[from] reqwest::Error),
}

/// Errors raised when attempting to load the user's configuration files.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    /// Error due to requested profile missing in the user's configuration.
    #[error("Expected profile {0} in settings.profiles but it didn't exist")]
    ProfileNotFound(String),
    /// Error due to authentication server missing in the user's configuration.
    #[error("Expected auth server {0} in settings.auth_servers but it didn't exist")]
    AuthServerNotFound(String),
    /// Error due to failing to find the user's home directory.
    #[error("Failed to determine home directory. You can use an explicit path by setting the {env} environment variable")]
    HomeDirError {
        /// Environment variable to set to configure the home directory.
        env: String,
    },
    /// Error due to failing to open a file.
    #[error("Could not open file at {path}")]
    FileOpenError {
        /// Path to file that could not be opened.
        path: PathBuf,
        /// The source error.
        source: std::io::Error,
    },
    /// Error due to failing to parse a file.
    #[error("Could not parse file at {path}")]
    FileParseError {
        /// Path to file that could not be parsed.
        path: PathBuf,
        /// Source error.
        source: toml::de::Error,
    },
}

impl Configuration {
    /// Attempt to load config files from ~/.qcs and create a Configuration object
    /// for use with qcs-api.
    ///
    /// # Errors
    ///
    /// See [`LoadError`].
    pub async fn load() -> Result<Self, LoadError> {
        let (settings, secrets) = try_join(settings::load(), secrets::load()).await?;
        Self::new(settings, secrets)
    }

    /// Refresh the `access_token` and return a new `Configuration` if successful.
    ///
    /// # Errors
    ///
    /// See [`RefreshError`].
    pub async fn refresh(mut self) -> Result<Self, RefreshError> {
        let refresh_token = self.refresh_token.ok_or(RefreshError::NoRefreshToken)?;
        let token_url = format!("{}/v1/token", &self.auth_server.issuer);
        let data = TokenRequest::new(&self.auth_server.client_id, &refresh_token);
        let resp = self
            .api_config
            .client
            .post(token_url)
            .form(&data)
            .send()
            .await?;
        let response_data: TokenResponse = resp.error_for_status()?.json().await?;
        self.api_config.bearer_access_token = Some(response_data.access_token);
        self.refresh_token = Some(response_data.refresh_token);
        Ok(self)
    }

    fn new(settings: Settings, mut secrets: Secrets) -> Result<Self, LoadError> {
        let Settings {
            default_profile_name,
            mut profiles,
            mut auth_servers,
        } = settings;
        let profile_name = std::env::var(PROFILE_NAME_VAR).unwrap_or(default_profile_name);
        let profile = profiles
            .remove(&profile_name)
            .ok_or(LoadError::ProfileNotFound(profile_name))?;
        let auth_server = auth_servers
            .remove(&profile.auth_server_name)
            .ok_or_else(|| AuthServerNotFound(profile.auth_server_name.clone()))?;

        let credential = secrets.credentials.remove(&profile.credentials_name);
        let (access_token, refresh_token) = match credential {
            Some(secrets::Credential {
                token_payload: Some(token_payload),
            }) => (token_payload.access_token, token_payload.refresh_token),
            _ => (None, None),
        };

        Ok(Self {
            api_config: api::Configuration {
                base_path: profile.api_url,
                bearer_access_token: access_token,
                ..api::Configuration::default()
            },
            auth_server,
            refresh_token,
            quilc_url: profile.applications.pyquil.quilc_url,
            qvm_url: profile.applications.pyquil.qvm_url,
        })
    }
}

#[derive(Debug, Serialize, Copy, Clone, Eq, PartialEq)]
struct TokenRequest<'a> {
    grant_type: &'static str,
    client_id: &'a str,
    refresh_token: &'a str,
}

impl<'a> TokenRequest<'a> {
    fn new(client_id: &'a str, refresh_token: &'a str) -> TokenRequest<'a> {
        Self {
            grant_type: "refresh_token",
            client_id,
            refresh_token,
        }
    }
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
struct TokenResponse {
    refresh_token: String,
    access_token: String,
}

impl AsRef<api::Configuration> for Configuration {
    fn as_ref(&self) -> &api::Configuration {
        &self.api_config
    }
}

impl Default for Configuration {
    fn default() -> Self {
        let Pyquil { quilc_url, qvm_url } = Pyquil::default();
        Self {
            quilc_url,
            qvm_url,
            api_config: api::Configuration::default(),
            auth_server: AuthServer::default(),
            refresh_token: None,
        }
    }
}
