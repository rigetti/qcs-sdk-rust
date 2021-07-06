#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![forbid(unsafe_code)]

use futures::future::try_join;
use thiserror::Error;

use qcs_api::apis::configuration::Configuration;

mod secrets;
mod settings;

/// Attempt to load config files from ~/.qcs and create a Configuration object
/// for use with qcs-api.
///
/// # Errors
/// See [`ConfigError`] for details on individual errors that can occur.
pub async fn get_configuration() -> Result<Configuration, ConfigError> {
    let (settings, secrets) = try_join(settings::load(), secrets::load()).await?;
    let credential = secrets.credentials.get(&settings.default_profile_name);
    let access_token = match credential {
        Some(secrets::Credential {
            token_payload: Some(token_payload),
        }) => token_payload.access_token.as_ref().cloned(),
        _ => None,
    };

    let mut configuration = Configuration::new();
    configuration.bearer_access_token = access_token;
    Ok(configuration)
}

/// Errors that can occur when attempting to load configuration.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// There was a problem loading the settings file.
    #[error("Could not load settings")]
    SettingsError(#[from] settings::Error),
    /// There was a problem loading the secrets file.
    #[error("Could not load secrets")]
    SecretsError(#[from] secrets::Error),
}
