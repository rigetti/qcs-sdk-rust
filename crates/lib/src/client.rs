//! This module provides methods for getting clients for the
//! desired API (e.g. `gRPC` or `OpenAPI`) and will properly
//! initialize those clients (e.g. with authentication metadata).

use std::time::Duration;

use qcs_api_client_common::configuration::{ClientConfiguration, RefreshError};
use qcs_api_client_grpc::{
    channel::{get_channel, parse_uri, wrap_channel_with, RefreshService},
    services::translation::translation_client::TranslationClient,
};
use qcs_api_client_openapi::apis::configuration::Configuration as OpenApiConfiguration;
use tonic::transport::Channel;
use tonic::Status;

pub use qcs_api_client_common::configuration::LoadError;
pub use qcs_api_client_grpc::channel::Error as GrpcError;
pub use qcs_api_client_openapi::apis::Error as OpenApiError;

/// A type alias for the underlying gRPC connection used by all gRPC clients within this library.
/// It is public so that users can create gRPC clients with different APIs using a "raw" connection
/// initialized by this library. This ensures that the exact Tonic version used for such clients
/// matches what this library uses.
pub type GrpcConnection = RefreshService<Channel, ClientConfiguration>;

/// TODO: make configurable at the client level.
/// <https://github.com/rigetti/qcs-sdk-rust/issues/239>
pub(crate) static DEFAULT_HTTP_API_TIMEOUT: Duration = Duration::from_secs(10);

/// A client providing helper functionality for accessing QCS APIs
#[derive(Debug, Clone, Default)]
pub struct Qcs {
    config: ClientConfiguration,
}

impl Qcs {
    /// Create a [`Qcs`] and initialize it with the user's default [`ClientConfiguration`]
    pub async fn load() -> Self {
        let config = if let Ok(config) = ClientConfiguration::load_default().await {
            config
        } else {
            #[cfg(feature = "tracing")]
            tracing::info!(
                "No QCS client configuration found. QPU data and QCS will be inaccessible and only generic QVMs will be available for execution"
            );
            ClientConfiguration::default()
        };
        Self::with_config(config)
    }

    /// Create a [`Qcs`] and initialize it with the given [`ClientConfiguration`]
    #[must_use]
    pub fn with_config(config: ClientConfiguration) -> Self {
        Self { config }
    }

    /// Create a [`Qcs`] and initialized with the given `profile`.
    ///
    /// # Errors
    ///
    /// A [`LoadError`] will be returned if QCS credentials are
    /// not correctly configured or the given profile is not defined.
    pub async fn with_profile(profile: String) -> Result<Qcs, LoadError> {
        ClientConfiguration::load_profile(profile)
            .await
            .map(Self::with_config)
    }

    /// Return a reference to the underlying [`ClientConfiguration`] with all settings parsed and resolved from configuration sources.
    #[must_use]
    pub fn get_config(&self) -> &ClientConfiguration {
        &self.config
    }

    pub(crate) fn get_openapi_client(&self) -> OpenApiConfiguration {
        OpenApiConfiguration::with_qcs_config(self.get_config().clone())
    }

    pub(crate) fn get_translation_client(
        &self,
    ) -> Result<TranslationClient<GrpcConnection>, GrpcError<RefreshError>> {
        self.get_translation_client_with_endpoint(self.get_config().grpc_api_url())
    }

    pub(crate) fn get_translation_client_with_endpoint(
        &self,
        translation_grpc_endpoint: &str,
    ) -> Result<TranslationClient<GrpcConnection>, GrpcError<RefreshError>> {
        let uri = parse_uri(translation_grpc_endpoint)?;
        let channel = get_channel(uri)?;
        let service = wrap_channel_with(channel, self.get_config().clone());
        Ok(TranslationClient::new(service))
    }
}

/// Errors that may occur while trying to use a `gRPC` client
#[derive(Debug, thiserror::Error)]
pub enum GrpcClientError {
    /// Error due to failure during request
    #[error("Call failed during gRPC request: {0}")]
    RequestFailed(#[from] Status),

    /// Error due to response body missing required data
    #[error("Response body had missing data: {0}")]
    ResponseEmpty(String),

    /// Error due to `gRPC` error
    #[error("gRPC error: {0}")]
    GrpcError(#[from] GrpcError<RefreshError>),
}

/// Errors that may occur while trying to use an `OpenAPI` client
#[derive(Debug, thiserror::Error)]
pub enum OpenApiClientError<T> {
    /// Error due to request failure
    #[error("Call failed during http request: {0}")]
    RequestFailed(#[from] OpenApiError<T>),

    /// Error due to empty response
    #[error("Response value was empty: {0}")]
    ResponseEmpty(String),
}
