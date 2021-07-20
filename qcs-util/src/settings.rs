use std::collections::HashMap;

use eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};

use crate::path::path_from_env_or_home;

const VAR: &str = "QCS_SETTINGS_FILE_PATH";

pub(crate) async fn load() -> Result<Settings> {
    let path = path_from_env_or_home(VAR, "settings.toml")
        .wrap_err("When determining settings config path")?;
    let content = tokio::fs::read_to_string(&path)
        .await
        .wrap_err_with(|| format!("When reading settings from {}", path.to_string_lossy()))?;
    Ok(toml::from_str(&content)
        .wrap_err_with(|| format!("When parsing settings from {}", path.to_string_lossy()))?)
}

#[cfg(test)]
mod describe_load {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;

    #[tokio::test]
    async fn it_returns_default_if_missing_path() {
        std::env::set_var(VAR, "/blah/doesnt_exist.toml");

        let settings = load().await;

        assert!(settings.is_err())
    }

    #[tokio::test]
    async fn it_loads_from_env_var_path() {
        let mut file = NamedTempFile::new().expect("Failed to create temporary settings file");
        let mut settings = Settings::default();
        settings.default_profile_name = "THIS IS A TEST".to_string();
        let settings_string =
            toml::to_string(&settings).expect("Could not serialize test settings");
        file.write(settings_string.as_bytes())
            .expect("Failed to write test settings");
        std::env::set_var(VAR, file.path());

        let loaded = load().await.expect("Failed to load settings");

        assert_eq!(settings, loaded)
    }
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
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

#[derive(Deserialize, Debug, PartialEq, Serialize)]
pub(crate) struct Profile {
    /// URL of the QCS API to use for all API calls
    pub api_url: String,
    auth_server_name: String,
    credentials_name: String,
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

#[derive(Deserialize, Debug, Default, PartialEq, Serialize)]
pub(crate) struct Applications {
    pub pyquil: Pyquil,
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
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

#[derive(Deserialize, Debug, PartialEq, Serialize)]
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
