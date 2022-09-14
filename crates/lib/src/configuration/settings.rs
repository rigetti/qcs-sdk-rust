use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::path::path_from_env_or_home;
use super::LoadError;

/// Setting the `QCS_SETTINGS_FILE_PATH` environment variable will change which file is used for loading settings
pub const SETTINGS_PATH_VAR: &str = "QCS_SETTINGS_FILE_PATH";

pub(crate) async fn load() -> Result<Settings, LoadError> {
    let path = path_from_env_or_home(SETTINGS_PATH_VAR, "settings.toml")?;
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

    use crate::configuration::{settings::load, SETTINGS_PATH_VAR};

    use super::Settings;

    #[tokio::test]
    async fn it_returns_default_if_missing_path() {
        std::env::set_var(SETTINGS_PATH_VAR, "/blah/doesnt_exist.toml");

        let settings = load().await;

        std::env::remove_var(SETTINGS_PATH_VAR);

        assert!(settings.is_err());
    }

    #[tokio::test]
    async fn it_loads_from_env_var_path() {
        let mut file = NamedTempFile::new().expect("Failed to create temporary settings file");
        let settings = Settings {
            default_profile_name: "THIS IS A TEST".to_string(),
            ..Default::default()
        };
        let settings_string =
            toml::to_string(&settings).expect("Could not serialize test settings");
        let _ = file
            .write(settings_string.as_bytes())
            .expect("Failed to write test settings");
        std::env::set_var(SETTINGS_PATH_VAR, file.path());

        let loaded = load().await.expect("Failed to load settings");

        assert_eq!(settings, loaded);
    }
}

#[derive(Deserialize, Debug, PartialEq, Serialize, Clone, Eq)]
pub(crate) struct Settings {
    /// Which profile to select settings from when none is specified.
    pub default_profile_name: String,
    /// All available configuration profiles, keyed by profile name.
    #[serde(default = "default_profiles")]
    pub profiles: HashMap<String, Profile>,
    #[serde(default)]
    pub(crate) auth_servers: HashMap<String, AuthServer>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_profile_name: "default".to_string(),
            profiles: default_profiles(),
            auth_servers: default_auth_servers(),
        }
    }
}

fn default_profiles() -> HashMap<String, Profile> {
    let mut map = HashMap::with_capacity(1);
    map.insert("default".to_string(), Profile::default());
    map
}

fn default_auth_servers() -> HashMap<String, AuthServer> {
    let mut map = HashMap::with_capacity(1);
    map.insert("default".to_string(), AuthServer::default());
    map
}

#[derive(Deserialize, Debug, PartialEq, Serialize, Clone, Eq)]
pub(crate) struct Profile {
    /// URL of the QCS API to use for all API calls
    pub api_url: String,
    pub auth_server_name: String,
    pub credentials_name: String,
    #[serde(default)]
    pub applications: Applications,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            api_url: "https://api.qcs.rigetti.com".to_string(),
            auth_server_name: "default".to_string(),
            credentials_name: "default".to_string(),
            applications: Applications::default(),
        }
    }
}

#[derive(Deserialize, Debug, Default, PartialEq, Serialize, Clone, Eq)]
pub(crate) struct Applications {
    pub pyquil: Pyquil,
}

#[derive(Deserialize, Debug, PartialEq, Serialize, Clone, Eq)]
pub(crate) struct Pyquil {
    pub qvm_url: String,
    pub quilc_url: String,
}

impl Default for Pyquil {
    fn default() -> Self {
        Self {
            qvm_url: "http://127.0.0.1:5000".to_string(),
            quilc_url: "tcp://127.0.0.1:5555".to_string(),
        }
    }
}

#[derive(Clone, Deserialize, Debug, PartialEq, Serialize, Eq)]
pub(crate) struct AuthServer {
    pub(crate) client_id: String,
    pub(crate) issuer: String,
}

impl Default for AuthServer {
    fn default() -> Self {
        Self {
            client_id: "0oa3ykoirzDKpkfzk357".to_string(),
            issuer: "https://auth.qcs.rigetti.com/oauth2/aus8jcovzG0gW2TUG355".to_string(),
        }
    }
}
