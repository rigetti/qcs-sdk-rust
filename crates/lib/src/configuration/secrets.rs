use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::configuration::LoadError;

use super::path::path_from_env_or_home;

/// Setting the `QCS_SECRETS_FILE_PATH` environment variable will change which file is used for loading secrets
pub const SECRETS_PATH_VAR: &str = "QCS_SECRETS_FILE_PATH";

pub(crate) async fn load() -> Result<Secrets, LoadError> {
    let path = path_from_env_or_home(SECRETS_PATH_VAR, "secrets.toml")?;
    let content =
        tokio::fs::read_to_string(&path)
            .await
            .map_err(|source| LoadError::FileOpenError {
                path: path.clone(),
                source,
            })?;
    toml::from_str(&content).map_err(|source| LoadError::FileParseError { path, source })
}

#[cfg(test)]
mod describe_load {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::{load, Credential, Secrets, SECRETS_PATH_VAR};

    #[tokio::test]
    async fn it_returns_default_if_missing_path() {
        std::env::set_var(SECRETS_PATH_VAR, "/blah/doesnt_exist.toml");

        let settings = load().await;

        std::env::remove_var(SECRETS_PATH_VAR);
        assert!(settings.is_err());
    }

    #[tokio::test]
    async fn it_loads_from_env_var_path() {
        let mut file = NamedTempFile::new().expect("Failed to create temporary settings file");
        let mut secrets = Secrets::default();
        secrets
            .credentials
            .insert("test".to_string(), Credential::default());
        let secrets_string = toml::to_string(&secrets).expect("Could not serialize test settings");
        let _ = file
            .write(secrets_string.as_bytes())
            .expect("Failed to write test settings");
        std::env::set_var(SECRETS_PATH_VAR, file.path());

        let loaded = load().await.expect("Failed to load secrets");

        assert_eq!(secrets, loaded);
    }
}

#[derive(Deserialize, Debug, PartialEq, Serialize, Clone, Eq)]
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

#[derive(Deserialize, Debug, Default, PartialEq, Serialize, Clone, Eq)]
pub(crate) struct Credential {
    pub token_payload: Option<TokenPayload>,
}

#[derive(Deserialize, Debug, Default, PartialEq, Serialize, Clone, Eq)]
pub(crate) struct TokenPayload {
    pub refresh_token: Option<String>,
    pub access_token: Option<String>,
    scope: Option<String>,
    expires_in: Option<u32>,
    id_token: Option<String>,
    token_type: Option<String>,
}
