//! This module provides methods for getting clients for the
//! desired API (e.g. ``gRPC`` or ``OpenAPI``) and will properly
//! initialize those clients (e.g. with authentication metadata).

use qcs_api_client_common::ClientConfiguration;
use qcs_api_client_grpc::{
    channel::{get_channel, parse_uri, wrap_channel_with, RefreshService},
    services::{
        controller::controller_client::ControllerClient,
        translation::translation_client::TranslationClient,
    },
};
use qcs_api_client_openapi::apis::{
    configuration::Configuration as OpenApiConfiguration,
    endpoints_api::{get_default_endpoint, GetDefaultEndpointError},
};
use tonic::transport::{Channel, Uri};
use tonic::Status;

pub use qcs_api_client_common::configuration::LoadError;
pub use qcs_api_client_grpc::channel::Error as GrpcError;
pub use qcs_api_client_openapi::apis::Error as OpenApiError;

/// A client providing helper functionality for accessing QCS APIs
#[derive(Clone, Default)]
pub struct Qcs {
    config: ClientConfiguration,
}

impl Qcs {
    /// Create a [`Qcs`] and initialize it with the user's default [`ClientConfiguration`]
    pub async fn load() -> Result<Self, LoadError> {
        ClientConfiguration::load().await.map(Self::with_config)
    }

    /// Create a [`Qcs`] and initialize it with the given [`ClientConfiguration`]
    #[must_use]
    pub fn with_config(config: ClientConfiguration) -> Self {
        Self { config }
    }

    pub(crate) fn get_config(&self) -> ClientConfiguration {
        self.config.clone()
    }

    pub(crate) async fn get_controller_client(
        &self,
        quantum_processor_id: &str,
    ) -> Result<ControllerClient<RefreshService<Channel>>, GrpcEndpointError> {
        self.get_controller_endpoint(quantum_processor_id)
            .await
            .map(get_channel)
            .map(|channel| wrap_channel_with(channel, self.get_config()))
            .map(ControllerClient::new)
    }

    pub(crate) fn get_openapi_client(&self) -> OpenApiConfiguration {
        OpenApiConfiguration::with_qcs_config(self.get_config())
    }

    pub(crate) fn get_translation_client(
        &self,
    ) -> Result<TranslationClient<RefreshService<Channel>>, GrpcError> {
        self.get_translation_client_with_endpoint(self.get_config().grpc_api_url())
    }

    pub(crate) fn get_translation_client_with_endpoint(
        &self,
        translation_grpc_endpoint: &str,
    ) -> Result<TranslationClient<RefreshService<Channel>>, GrpcError> {
        parse_uri(translation_grpc_endpoint)
            .map(get_channel)
            .map(|channel| wrap_channel_with(channel, self.get_config()))
            .map(TranslationClient::new)
    }

    async fn get_controller_endpoint(
        &self,
        quantum_processor_id: &str,
    ) -> Result<Uri, GrpcEndpointError> {
        let default_endpoint =
            get_default_endpoint(&self.get_openapi_client(), quantum_processor_id).await?;
        let addresses = default_endpoint.addresses.as_ref();
        let grpc_address = addresses.grpc.as_ref();

        grpc_address
            .ok_or_else(|| GrpcEndpointError::NoEndpoint(quantum_processor_id.into()))
            .map(|v| parse_uri(v).map_err(GrpcEndpointError::BadUri))?
    }
}

/// Errors that may occur while trying to resolve a ``gRPC`` endpoint
#[derive(Debug, thiserror::Error)]
pub enum GrpcEndpointError {
    /// Error due to a malformed URI
    #[error("Malformed URI for endpoint: {0}")]
    BadUri(#[from] GrpcError),

    /// Error due to failure to get endpoint for quantum processor
    #[error("Failed to get endpoint for quantum processor: {0}")]
    RequestFailed(#[from] OpenApiError<GetDefaultEndpointError>),

    /// Error due to missing gRPC endpoint for quantum processor
    #[error("Missing gRPC endpoint for quantum processor {0}")]
    NoEndpoint(String),
}

/// Errors that may occur while trying to use a ``gRPC`` client
#[derive(Debug, thiserror::Error)]
pub enum GrpcClientError {
    /// Error due to failure to resolve the endpoint
    #[error("Failed to resolve the gRPC endoint: {0}")]
    EndpointNotResolved(#[from] GrpcEndpointError),

    /// Error due to failure during request
    #[error("Call failed during gRPC request: {0}")]
    RequestFailed(#[from] Status),

    /// Error due to response body missing required data
    #[error("Response body had missing data: {0}")]
    ResponseEmpty(String),

    /// Error due to ``gRPC`` error
    #[error("gRPC error: {0}")]
    GrpcError(#[from] GrpcError),
}

/// Errors that may occur while trying to use a [`OpenAPI`] client
#[derive(Debug, thiserror::Error)]
pub enum OpenApiClientError<T> {
    /// Error due to request failure
    #[error("Call failed during http request: {0}")]
    RequestFailed(#[from] OpenApiError<T>),

    /// Error due to empty response
    #[error("Response value was empty: {0}")]
    ResponseEmpty(String),
}
