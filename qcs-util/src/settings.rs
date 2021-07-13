use std::collections::HashMap;

use serde::Deserialize;

pub(crate) async fn load() -> Settings {
    _load().await.unwrap_or_else(|_| Settings::default())
}

async fn _load() -> Result<Settings, Error> {
    let home = dirs::home_dir().ok_or(Error::HomeDirectory)?;
    let content = tokio::fs::read_to_string(home.join(".qcs").join("settings.toml")).await?;
    Ok(toml::from_str(&content)?)
}

/// Errors that can occur when attempting to load settings.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The default settings path is in the home directory. It's possible, depending on platform,
    /// that the home directory couldn't be determined.
    #[error("Could not determine a home directory to read config from")]
    HomeDirectory,
    /// There was no `.qcs/settings.toml` file in the home directory.
    #[error("Could read ~/.qcs/settings.toml")]
    Path(#[from] std::io::Error),
    /// The settings file existed but could not be parsed.
    #[error("Could not parse settings file")]
    Parse(#[from] toml::de::Error),
}

#[derive(Deserialize, Debug)]
pub(crate) struct Settings {
    /// Which profile to select settings from when none is specified.
    pub default_profile_name: String,
    /// All available configuration profiles, keyed by profile name.
    #[serde(default = "default_profiles")]
    pub profiles: HashMap<String, Profile>,
    #[serde(default)]
    auth_servers: HashMap<String, AuthServer>,
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

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug, Default)]
pub(crate) struct Applications {
    pub pyquil: Pyquil,
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
struct AuthServer {
    client_id: String,
    issuer: String,
}

impl Default for AuthServer {
    fn default() -> Self {
        Self {
            client_id: "0oa3ykoirzDKpkfzk357".to_string(),
            issuer: "https://auth.qcs.rigetti.com/oauth2/aus8jcovzG0gW2TUG355".to_string(),
        }
    }
}
