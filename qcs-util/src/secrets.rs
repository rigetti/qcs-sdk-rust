use std::collections::HashMap;

use serde::Deserialize;

use crate::ConfigError;

pub(crate) async fn load() -> Result<Secrets, ConfigError> {
    _load().await.map_err(ConfigError::from)
}

async fn _load() -> Result<Secrets, Error> {
    let home = dirs::home_dir().ok_or(Error::HomeDirectory)?;
    let content = tokio::fs::read_to_string(home.join(".qcs").join("secrets.toml")).await?;
    Ok(toml::from_str(&content)?)
}

/// Errors that can occur when attempting to load secrets.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The default settings path is in the home directory. It's possible, depending on platform,
    /// that the home directory couldn't be determined.
    #[error("Could not determine a home directory to read config from")]
    HomeDirectory,
    /// There was no `.qcs/secrets.toml` file in the home directory.
    #[error("Could read ~/.qcs/secrets.toml")]
    Path(#[from] std::io::Error),
    /// The secrets file existed but could not be parsed.
    #[error("Could not parse secrets file")]
    Parse(#[from] toml::de::Error),
}

#[derive(Deserialize, Debug)]
pub(crate) struct Secrets {
    pub credentials: HashMap<String, Credential>,
}

impl Default for Secrets {
    fn default() -> Self {
        Self {
            credentials: default_credentials(),
        }
    }
}

fn default_credentials() -> HashMap<String, Credential> {
    let mut map = HashMap::with_capacity(1);
    map.insert("default".to_string(), Credential::default());
    map
}

#[derive(Deserialize, Debug, Default)]
pub(crate) struct Credential {
    pub token_payload: Option<TokenPayload>,
}

#[derive(Deserialize, Debug, Default)]
pub(crate) struct TokenPayload {
    refresh_token: Option<String>,
    pub access_token: Option<String>,
    scope: Option<String>,
    expires_in: Option<u32>,
    id_token: Option<String>,
    token_type: Option<String>,
}
