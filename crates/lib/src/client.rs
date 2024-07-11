//! This module provides methods for getting clients for the
//! desired API (e.g. `gRPC` or `OpenAPI`) and will properly
//! initialize those clients (e.g. with authentication metadata).

use std::time::Duration;

use qcs_api_client_common::configuration::{ClientConfiguration, TokenError};
#[cfg(feature = "grpc-web")]
use qcs_api_client_grpc::tonic::{wrap_channel_with_grpc_web, GrpcWebWrapperLayerService};
use qcs_api_client_grpc::{
    services::translation::translation_client::TranslationClient,
    tonic::{
        get_channel, parse_uri, wrap_channel_with, wrap_channel_with_retry, RefreshService,
        RetryService,
    },
};
use qcs_api_client_openapi::apis::configuration::Configuration as OpenApiConfiguration;
use tonic::transport::Channel;
use tonic::Status;

pub use qcs_api_client_common::configuration::LoadError;
pub use qcs_api_client_grpc::tonic::Error as GrpcError;
pub use qcs_api_client_openapi::apis::Error as OpenApiError;

const DEFAULT_MAX_MESSAGE_ENCODING_SIZE: usize = 50 * 1024 * 1024;
const DEFAULT_MAX_MESSAGE_DECODING_SIZE: usize = 50 * 1024 * 1024;

/// A type alias for the underlying gRPC connection used by all gRPC clients within this library.
/// It is public so that users can create gRPC clients with different APIs using a "raw" connection
/// initialized by this library. This ensures that the exact Tonic version used for such clients
/// matches what this library uses.
#[cfg(not(feature = "grpc-web"))]
pub type GrpcConnection = RetryService<RefreshService<Channel, ClientConfiguration>>;

/// A type alias for the underlying gRPC connection used by all gRPC clients within this library.
/// It is public so that users can create gRPC clients with different APIs using a "raw" connection
/// initialized by this library. This ensures that the exact Tonic version used for such clients
/// matches what this library uses.
#[cfg(feature = "grpc-web")]
pub type GrpcConnection =
    GrpcWebWrapperLayerService<RetryService<RefreshService<Channel, ClientConfiguration>>>;

/// TODO: make configurable at the client level.
/// <https://github.com/rigetti/qcs-sdk-rust/issues/239>
pub(crate) static DEFAULT_HTTP_API_TIMEOUT: Duration = Duration::from_secs(10);

/// A client providing helper functionality for accessing QCS APIs
#[derive(Debug, Clone)]
pub struct Qcs {
    config: ClientConfiguration,
}

impl Qcs {
    /// Create a [`Qcs`] and initialize it with the user's default [`ClientConfiguration`]
    #[must_use]
    pub fn load() -> Self {
        if let Ok(config) = ClientConfiguration::load_default() {
            Self::with_config(config)
        } else {
            #[cfg(feature = "tracing")]
            tracing::info!(
                "No QCS client configuration found. QPU data and QCS will be inaccessible and only generic QVMs will be available for execution"
            );
            Self::default()
        }
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
    pub fn with_profile(profile: String) -> Result<Qcs, LoadError> {
        ClientConfiguration::load_profile(profile).map(Self::with_config)
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
    ) -> Result<TranslationClient<GrpcConnection>, GrpcError<TokenError>> {
        self.get_translation_client_with_endpoint(self.get_config().grpc_api_url())
    }

    pub(crate) fn get_translation_client_with_endpoint(
        &self,
        translation_grpc_endpoint: &str,
    ) -> Result<TranslationClient<GrpcConnection>, GrpcError<TokenError>> {
        let uri = parse_uri(translation_grpc_endpoint)?;
        let channel = get_channel(uri)?;
        let service =
            wrap_channel_with_retry(wrap_channel_with(channel, self.get_config().clone()));
        #[cfg(feature = "grpc-web")]
        let service = wrap_channel_with_grpc_web(service);
        Ok(TranslationClient::new(service)
            .max_encoding_message_size(DEFAULT_MAX_MESSAGE_ENCODING_SIZE)
            .max_decoding_message_size(DEFAULT_MAX_MESSAGE_DECODING_SIZE))
    }
}

impl Default for Qcs {
    fn default() -> Self {
        Self::with_config(
            ClientConfiguration::builder()
                .build()
                .expect("builder should be valid with all defaults"),
        )
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
    GrpcError(#[from] GrpcError<TokenError>),
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
