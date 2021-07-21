//! This module is used for loading configuration that will be used to connect either to real QPUs
//! (and supporting services) or the QVM. By default, all settings are loaded from files located
//! under your home directory in a `.qcs` folder. `settings.toml` will be used to load general
//! settings (e.g. which URLs to connect to) and `secrets.toml` will be used to load tokens for
//! authentication. Both "settings" and "secrets" files should contain profiles. The
//! `default_profile_name` in settings sets the profile to be used when there is no override. You
//! can set the [`PROFILE_NAME_VAR`] to select a different profile. You can also use
//! [`SECRETS_PATH_VAR`] and [`SETTINGS_PATH_VAR`] to change which files are loaded.

use eyre::{eyre, Result, WrapErr};
use futures::future::try_join;
use serde::{Deserialize, Serialize};

use qcs_api::apis::configuration as api;
use secrets::Secrets;
pub use secrets::SECRETS_PATH_VAR;
pub use settings::SETTINGS_PATH_VAR;
use settings::{AuthServer, Pyquil, Settings};

mod path;
mod secrets;
mod settings;

/// All the config data that's parsed from config sources
pub(crate) struct Configuration {
    api_config: api::Configuration,
    auth_server: AuthServer,
    refresh_token: Option<String>,
    pub quilc_url: String,
    pub qvm_url: String,
}

/// Setting this environment variable will change which profile is used from the loaded config files
pub const PROFILE_NAME_VAR: &str = "QCS_PROFILE_NAME";

impl Configuration {
    /// Attempt to load config files from ~/.qcs and create a Configuration object
    /// for use with qcs-api.
    ///
    /// # Errors
    /// Errors are human-readable (using eyre) since generally they aren't recoverable at runtime.
    /// Usually this function will error if one of the config files is missing or malformed.
    pub(crate) async fn load() -> Result<Self> {
        let (settings, secrets) = try_join(settings::load(), secrets::load()).await?;
        Self::new(settings, secrets)
    }

    /// Refresh the `access_token` and return a new `Configuration` if successful.
    ///
    /// # Errors
    ///
    /// 1. There is no `refresh_token` set, so no new `access_token` can be fetched.
    /// 2. Could not reach the configured auth server.
    /// 3. The response from the auth server was invalid.
    pub(crate) async fn refresh(mut self) -> Result<Self> {
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
