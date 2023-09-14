//! Produce diagnostic information about the crate and its runtime environment in order to aid
//! in debugging and remote user support.

use std::{borrow::Cow, time::Duration};

use qcs_api_client_openapi::models::User;

use crate::{
    build_info,
    client::Qcs,
    compiler::{quilc::Client as _, rpcq},
    qvm::{self, Client as _, QvmOptions},
};

/// Collect package diagnostics in string form
pub async fn get_report() -> String {
    Diagnostics::gather().await.to_string()
}

/// Diagnostic information representing the environment in which this crate
/// was built and is executed, for use in diagnosing unexpected and incorrect
/// behavior.
#[derive(Debug)]
struct Diagnostics {
    /// The version of this crate    
    version: String,

    rust_version: String,

    /// The features with which this crate was compiled
    features: Vec<&'static str>,

    qcs: QcsApiDiagnostics,
    quilc: QuilcDiagnostics,
    qvm: QvmDiagnostics,
}

impl Diagnostics {
    async fn gather() -> Self {
        let client = Qcs::load().await;

        let (qcs, qvm) = futures::future::join(
            QcsApiDiagnostics::gather(&client),
            QvmDiagnostics::gather(&client),
        )
        .await;

        Self {
            version: build_info::PKG_VERSION.to_owned(),
            rust_version: build_info::RUSTC_VERSION.to_owned(),
            features: build_info::FEATURES.to_vec(),
            qcs,
            quilc: QuilcDiagnostics::gather(&client),
            qvm,
        }
    }
}

impl std::fmt::Display for Diagnostics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "qcs-sdk-rust version: {}", self.version)?;
        writeln!(f, "rustc version: {}", self.rust_version)?;
        writeln!(f, "features: {}", self.features.join(", "))?;
        writeln!(f, "qcs:")?;
        writeln!(f, "  address: {}", self.qcs.address)?;
        writeln!(f, "  network reachable: {}", self.qcs.network_reachable)?;
        writeln!(f, "  authentication: {}", self.qcs.authentication)?;
        writeln!(f, "quilc:")?;
        writeln!(f, "  address: {}", self.quilc.address)?;
        writeln!(
            f,
            "  version: {}",
            format_option(self.quilc.version.as_ref())
        )?;
        writeln!(f, "  available: {}", self.quilc.available)?;
        writeln!(f, "qvm:")?;
        writeln!(f, "  address: {}", self.qvm.address)?;
        writeln!(f, "  version: {}", format_option(self.qvm.version.as_ref()))?;
        writeln!(f, "  available: {}", self.qvm.available)?;
        Ok(())
    }
}

#[derive(Debug)]
struct QcsApiDiagnostics {
    address: String,
    network_reachable: bool,
    authentication: QcsApiAuthenticationResult,
}

#[derive(Debug)]
enum QcsApiAuthenticationResult {
    Success(User),
    Failure {
        status_code: Option<reqwest::StatusCode>,
        error: String,
    },
}

impl std::fmt::Display for QcsApiAuthenticationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QcsApiAuthenticationResult::Success(user) => {
                write!(
                    f,
                    "{} / email {}\n  ^^^ for your security, remove this line before posting publicly! ^^^",
                    user.idp_id,
                    format_option(user.profile.as_ref().map(|profile| &profile.email))
                )?;
            }
            QcsApiAuthenticationResult::Failure { status_code, error } => {
                write!(f, "failure: ")?;
                match status_code {
                    Some(status_code) => {
                        write!(f, " (status: {status_code}): ")?;
                    }
                    None => {
                        write!(f, " (no status code): ")?;
                    }
                }
                write!(f, "{error}")?;
            }
        }
        Ok(())
    }
}

impl QcsApiDiagnostics {
    async fn gather(client: &Qcs) -> Self {
        let configuration = client.get_config();
        let address = configuration.api_url().to_string();

        let network_reachable = reqwest::get(&address).await.is_ok();
        let client = qcs_api_client_openapi::apis::configuration::Configuration::with_qcs_config(
            configuration.clone(),
        );

        let authentication =
            match qcs_api_client_openapi::apis::authentication_api::auth_get_user(&client).await {
                Ok(response) => QcsApiAuthenticationResult::Success(response),
                Err(error) => QcsApiAuthenticationResult::Failure {
                    status_code: error.status_code(),
                    error: error.to_string(),
                },
            };

        Self {
            address,
            network_reachable,
            authentication,
        }
    }
}

#[derive(Debug)]
struct QuilcDiagnostics {
    address: String,
    version: Option<String>,
    available: bool,
}

impl QuilcDiagnostics {
    fn gather(client: &Qcs) -> Self {
        let address = client.get_config().quilc_url().to_string();
        match rpcq::Client::new(&address) {
            Ok(client) => {
                let (version, available) = match client.get_version_info() {
                    Ok(version) => (Some(version), true),
                    Err(_) => (None, false),
                };

                Self {
                    address,
                    version,
                    available,
                }
            }
            Err(_) => Self {
                address,
                version: None,
                available: false,
            },
        }
    }
}

#[derive(Debug)]
struct QvmDiagnostics {
    address: String,
    version: Option<String>,
    available: bool,
}

impl QvmDiagnostics {
    async fn gather(client: &Qcs) -> Self {
        let options = QvmOptions {
            timeout: Some(Duration::from_secs(1)),
        };
        let qvm_client = qvm::http::HttpClient::from(client);
        let (version, available) = match qvm_client.get_version_info(&options).await {
            Ok(version) => (Some(version), true),
            Err(_) => (None, false),
        };

        Self {
            address: qvm_client.qvm_url,
            version,
            available,
        }
    }
}

fn format_option<T>(value: Option<T>) -> Cow<'static, str>
where
    T: std::fmt::Display,
{
    match value {
        Some(value) => value.to_string().into(),
        None => "-".into(),
    }
}
