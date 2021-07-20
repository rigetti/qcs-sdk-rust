#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![forbid(unsafe_code)]

use eyre::{eyre, Result, WrapErr};
use futures::future::try_join;
use serde::{Deserialize, Serialize};

use qcs_api::apis::configuration as api;
pub use secrets::SECRETS_PATH_VAR;
pub use settings::SETTINGS_PATH_VAR;

use crate::secrets::Secrets;
use crate::settings::{AuthServer, Pyquil, Settings};

pub mod engagement;
mod path;
mod secrets;
mod settings;

/// Attempt to load config files from ~/.qcs and create a Configuration object
/// for use with qcs-api.
///
/// # Errors
/// See [`ConfigError`] for details on individual errors that can occur.
pub async fn get_configuration() -> Result<Configuration> {
    let (settings, secrets) = try_join(settings::load(), secrets::load()).await?;
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

pub const PROFILE_NAME_VAR: &str = "QCS_PROFILE_NAME";

impl Configuration {
    fn new(settings: Settings, mut secrets: Secrets) -> Result<Self> {
        let Settings {
            default_profile_name,
            mut profiles,
            mut auth_servers,
        } = settings;
        let profile_name = std::env::var(PROFILE_NAME_VAR).unwrap_or(default_profile_name);
        let profile = profiles.remove(&profile_name).ok_or_else(|| {
            eyre!(
                "Expected profile {} in settings.profiles but it didn't exist",
                profile_name
            )
        })?;
        let auth_server = auth_servers
            .remove(&profile.auth_server_name)
            .ok_or_else(|| {
                eyre!(
                    "Expected auth server {} in settings.auth_servers but it didn't exist",
                    &profile.auth_server_name
                )
            })?;

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

    /// Refresh the `access_token` and return a new `Configuration` if successful.
    ///
    /// # Errors
    ///
    /// 1. There is no `refresh_token` set, so no new `access_token` can be fetched.
    /// 2. Could not reach the configured auth server.
    /// 3. The response from the auth server was invalid.
    pub async fn refresh(mut self) -> Result<Self> {
        let refresh_token = self
            .refresh_token
            .ok_or_else(|| eyre!("No refresh token is in secrets"))?;
        let token_url = format!("{}/v1/token", &self.auth_server.issuer);
        let data = TokenRequest::new(&self.auth_server.client_id, &refresh_token);
        let resp = self
            .api_config
            .client
            .post(token_url)
            .form(&data)
            .send()
            .await
            .wrap_err("While requesting a new access token using refresh token")?;
        let response_data: TokenResponse = resp
            .error_for_status()?
            .json()
            .await
            .wrap_err("While decoding response from auth server")?;
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
