#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![forbid(unsafe_code)]

use futures::future::join;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use qcs_api::apis::configuration as api;

use crate::secrets::Secrets;
use crate::settings::{AuthServer, Pyquil, Settings};

pub mod engagement;
mod secrets;
mod settings;

/// Attempt to load config files from ~/.qcs and create a Configuration object
/// for use with qcs-api.
///
/// # Errors
/// See [`ConfigError`] for details on individual errors that can occur.
pub async fn get_configuration() -> Result<Configuration, ConfigError> {
    let (settings, secrets) = join(settings::load(), secrets::load()).await;
    Configuration::new(settings, secrets)
}

/// All the config data that's parsed from config sources
pub struct Configuration {
    api_config: api::Configuration,
    auth_server: AuthServer,
    refresh_token: Option<String>,
    pub quilc_url: String,
    pub qvm_url: String,
}

impl Configuration {
    fn new(settings: Settings, mut secrets: Secrets) -> Result<Self, ConfigError> {
        let Settings {
            default_profile_name: profile_name,
            mut profiles,
            mut auth_servers,
        } = settings;
        let profile = profiles.remove(&profile_name).ok_or(ConfigError::Profile)?;
        let auth_server = auth_servers
            .remove(&profile_name)
            .ok_or(ConfigError::Profile)?;

        let credential = secrets.credentials.remove(&profile_name);
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

    /// Refresh the `access_token` and return a new `Configuration` if successful.
    ///
    /// # Errors
    ///
    /// 1. There is no `refresh_token` set, so no new `access_token` can be fetched.
    /// 2. Could not reach the configured auth server.
    /// 3. The response from the auth server was invalid.
    pub async fn refresh(mut self) -> Result<Self, ConfigError> {
        let refresh_token = self.refresh_token.ok_or(ConfigError::RefreshToken)?;
        let token_url = format!("{}/v1/token", &self.auth_server.issuer);
        let data = TokenRequest::new(&self.auth_server.client_id, &refresh_token);
        let resp = self
            .api_config
            .client
            .post(token_url)
            .form(&data)
            .send()
            .await
            .map_err(|_| ConfigError::RefreshToken)?;
        let response_data: TokenResponse =
            resp.json().await.map_err(|_| ConfigError::RefreshToken)?;
        self.api_config.bearer_access_token = Some(response_data.access_token);
        self.refresh_token = Some(response_data.refresh_token);
        Ok(self)
    }
}

#[derive(Debug, Serialize)]
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

#[derive(Deserialize, Debug)]
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

/// Errors that can occur when attempting to load configuration.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// There was a problem loading the settings file.
    #[error("Could not load settings")]
    Settings(#[from] settings::Error),
    /// There was a problem loading the secrets file.
    #[error("Could not load secrets")]
    Secrets(#[from] secrets::Error),
    /// The specified `default_profile_name` in Settings did not correspond to a profile
    #[error("Default profile did not exist")]
    Profile,
    /// There was no refresh token configured, so a new access token could not be retrieved.
    #[error("No refresh token is configured")]
    RefreshToken,
}
