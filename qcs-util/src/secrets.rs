use std::collections::HashMap;

use eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};

use crate::path::path_from_env_or_home;

pub const SECRETS_PATH_VAR: &str = "QCS_SECRETS_FILE_PATH";

pub(crate) async fn load() -> Result<Secrets> {
    let path = path_from_env_or_home(SECRETS_PATH_VAR, "secrets.toml")
        .wrap_err("When determining secrets config path")?;
    let content = tokio::fs::read_to_string(&path)
        .await
        .wrap_err_with(|| format!("While reading secrets from {}", path.to_string_lossy()))?;
    Ok(toml::from_str(&content)
        .wrap_err_with(|| format!("While parsing secrets from {}", path.to_string_lossy()))?)
}

#[cfg(test)]
mod describe_load {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;

    #[tokio::test]
    async fn it_returns_default_if_missing_path() {
        std::env::set_var(SECRETS_PATH_VAR, "/blah/doesnt_exist.toml");

        let settings = load().await;

        assert!(settings.is_err())
    }

    #[tokio::test]
    async fn it_loads_from_env_var_path() {
        let mut file = NamedTempFile::new().expect("Failed to create temporary settings file");
        let mut secrets = Secrets::default();
        secrets
            .credentials
            .insert("test".to_string(), Credential::default());
        let secrets_string = toml::to_string(&secrets).expect("Could not serialize test settings");
        file.write(secrets_string.as_bytes())
            .expect("Failed to write test settings");
        std::env::set_var(SECRETS_PATH_VAR, file.path());

        let loaded = load().await.expect("Failed to load secrets");

        assert_eq!(secrets, loaded)
    }
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
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

#[derive(Deserialize, Debug, Default, PartialEq, Serialize)]
pub(crate) struct Credential {
    pub token_payload: Option<TokenPayload>,
}

#[derive(Deserialize, Debug, Default, PartialEq, Serialize)]
pub(crate) struct TokenPayload {
    pub refresh_token: Option<String>,
    pub access_token: Option<String>,
    scope: Option<String>,
    expires_in: Option<u32>,
    id_token: Option<String>,
    token_type: Option<String>,
}
