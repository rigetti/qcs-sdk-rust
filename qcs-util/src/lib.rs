#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![forbid(unsafe_code)]

use futures::future::try_join;
use thiserror::Error;

use crate::secrets::Secrets;
use crate::settings::{Pyquil, Settings};
use qcs_api::apis::configuration as api;

mod secrets;
mod settings;

/// Attempt to load config files from ~/.qcs and create a Configuration object
/// for use with qcs-api.
///
/// # Errors
/// See [`ConfigError`] for details on individual errors that can occur.
pub async fn get_configuration() -> Result<Configuration, ConfigError> {
    let (settings, secrets) = try_join(settings::load(), secrets::load()).await?;
    Configuration::new(settings, secrets)
}

/// All the config data that's parsed from config sources
pub struct Configuration {
    api_config: api::Configuration,
    pub quilc_url: String,
    pub qvm_url: String,
}

impl Configuration {
    fn new(settings: Settings, mut secrets: Secrets) -> Result<Self, ConfigError> {
        let Settings {
            default_profile_name: profile_name,
            mut profiles,
            ..
        } = settings;
        let profile = profiles.remove(&profile_name).ok_or(ConfigError::Profile)?;

        let credential = secrets.credentials.remove(&profile_name);
        let access_token = match credential {
            Some(secrets::Credential {
                token_payload: Some(token_payload),
            }) => token_payload.access_token,
            _ => None,
        };

        Ok(Self {
            api_config: api::Configuration {
                base_path: profile.api_url,
                bearer_access_token: access_token,
                ..api::Configuration::default()
            },
            quilc_url: profile.applications.pyquil.quilc_url,
            qvm_url: profile.applications.pyquil.qvm_url,
        })
    }
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
            api_config: api::Configuration::default(),
            quilc_url,
            qvm_url,
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
}
