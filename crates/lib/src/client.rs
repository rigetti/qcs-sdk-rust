//! This module provides methods for getting clients for the
//! desired API (e.g. `gRPC` or `OpenAPI`) and will properly
//! initialize those clients (e.g. with authentication metadata).

use std::time::Duration;

use qcs_api_client_common::configuration::{ClientConfiguration, RefreshError};
use qcs_api_client_grpc::{
    channel::{get_channel, parse_uri, wrap_channel_with, RefreshService},
    services::{
        controller::controller_client::ControllerClient,
        translation::translation_client::TranslationClient,
    },
};
use qcs_api_client_openapi::apis::{
    configuration::Configuration as OpenApiConfiguration,
    endpoints_api::{
        get_default_endpoint, get_endpoint, GetDefaultEndpointError, GetEndpointError,
    },
    quantum_processors_api::{
        list_quantum_processor_accessors, ListQuantumProcessorAccessorsError,
    },
};
use qcs_api_client_openapi::models::QuantumProcessorAccessorType;
use tonic::transport::{Channel, Uri};
use tonic::Status;

pub use qcs_api_client_common::configuration::LoadError;
pub use qcs_api_client_grpc::channel::Error as GrpcError;
pub use qcs_api_client_openapi::apis::Error as OpenApiError;

/// TODO: make configurable at the client level.
/// <https://github.com/rigetti/qcs-sdk-rust/issues/239>
pub(crate) static DEFAULT_HTTP_API_TIMEOUT: Duration = Duration::from_secs(10);

/// A client providing helper functionality for accessing QCS APIs
#[derive(Debug, Clone, Default)]
pub struct Qcs {
    config: ClientConfiguration,
    /// When enabled, default to Gateway service for execution. Fallback to QPU's default endpoint otherwise.
    use_gateway: bool,
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
        Self {
            config,
            use_gateway: true,
        }
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

    /// Enable or disable the use of Gateway service for execution
    #[must_use]
    pub fn with_use_gateway(mut self, use_gateway: bool) -> Self {
        self.use_gateway = use_gateway;
        self
    }

    /// Return a reference to the underlying [`ClientConfiguration`] with all settings parsed and resolved from configuration sources.
    #[must_use]
    pub fn get_config(&self) -> &ClientConfiguration {
        &self.config
    }

    pub(crate) async fn get_controller_client(
        &self,
        quantum_processor_id: &str,
    ) -> Result<ControllerClient<RefreshService<Channel, ClientConfiguration>>, GrpcEndpointError>
    {
        let uri = self.get_controller_endpoint(quantum_processor_id).await?;
        let channel = get_channel(uri).map_err(|err| GrpcEndpointError::GrpcError(err.into()))?;
        let service = wrap_channel_with(channel, self.get_config().clone());
        Ok(ControllerClient::new(service))
    }

    pub(crate) async fn get_controller_client_with_endpoint_id(
        &self,
        endpoint_id: &str,
    ) -> Result<ControllerClient<RefreshService<Channel, ClientConfiguration>>, GrpcEndpointError>
    {
        let uri = self.get_controller_endpoint_by_id(endpoint_id).await?;
        let channel = get_channel(uri).map_err(|err| GrpcEndpointError::GrpcError(err.into()))?;
        let service = wrap_channel_with(channel, self.get_config().clone());
        Ok(ControllerClient::new(service))
    }

    pub(crate) fn get_openapi_client(&self) -> OpenApiConfiguration {
        OpenApiConfiguration::with_qcs_config(self.get_config().clone())
    }

    pub(crate) fn get_translation_client(
        &self,
    ) -> Result<
        TranslationClient<RefreshService<Channel, ClientConfiguration>>,
        GrpcError<RefreshError>,
    > {
        self.get_translation_client_with_endpoint(self.get_config().grpc_api_url())
    }

    pub(crate) fn get_translation_client_with_endpoint(
        &self,
        translation_grpc_endpoint: &str,
    ) -> Result<
        TranslationClient<RefreshService<Channel, ClientConfiguration>>,
        GrpcError<RefreshError>,
    > {
        let uri = parse_uri(translation_grpc_endpoint)?;
        let channel = get_channel(uri)?;
        let service = wrap_channel_with(channel, self.get_config().clone());
        Ok(TranslationClient::new(service))
    }

    async fn get_controller_endpoint(
        &self,
        quantum_processor_id: &str,
    ) -> Result<Uri, GrpcEndpointError> {
        if self.use_gateway {
            let gateway = self.get_gateway_endpoint(quantum_processor_id).await;
            // when no gateway is available, we should fall through and attempt a direct connection
            if gateway.is_ok() {
                return gateway;
            }
        }
        self.get_controller_default_endpoint(quantum_processor_id)
            .await
    }

    /// Get address for direct connection to Controller, explicitly targeting an endpoint by ID.
    async fn get_controller_endpoint_by_id(
        &self,
        endpoint_id: &str,
    ) -> Result<Uri, GrpcEndpointError> {
        let endpoint = get_endpoint(&self.get_openapi_client(), endpoint_id).await?;
        let grpc_address = endpoint.addresses.grpc;

        grpc_address
            .ok_or_else(|| GrpcEndpointError::EndpointNotFound(endpoint_id.into()))
            .map(|v| parse_uri(&v).map_err(GrpcEndpointError::GrpcError))?
    }

    /// Get address for direction connection to Controller.
    async fn get_controller_default_endpoint(
        &self,
        quantum_processor_id: &str,
    ) -> Result<Uri, GrpcEndpointError> {
        let default_endpoint =
            get_default_endpoint(&self.get_openapi_client(), quantum_processor_id).await?;
        let addresses = default_endpoint.addresses.as_ref();
        let grpc_address = addresses.grpc.as_ref();
        grpc_address
            .ok_or_else(|| GrpcEndpointError::QpuEndpointNotFound(quantum_processor_id.into()))
            .map(|v| parse_uri(v).map_err(GrpcEndpointError::GrpcError))?
    }

    /// Get address for Gateway assigned to the provided `quantum_processor_id`, if one exists.
    async fn get_gateway_endpoint(
        &self,
        quantum_processor_id: &str,
    ) -> Result<Uri, GrpcEndpointError> {
        let mut gateways = Vec::new();
        let mut next_page_token = None;
        loop {
            let accessors = list_quantum_processor_accessors(
                &self.get_openapi_client(),
                quantum_processor_id,
                Some(100),
                next_page_token.as_deref(),
            )
            .await?;
            gateways.extend(accessors.accessors.into_iter().filter(|acc| {
                acc.live
                    // `as_deref` needed to work around the `Option<Box<_>>` type.
                    && acc.access_type.as_deref() == Some(&QuantumProcessorAccessorType::GatewayV1)
            }));
            next_page_token = accessors.next_page_token.clone();
            if next_page_token.is_none() {
                break;
            }
        }
        gateways.sort_by_key(|acc| acc.rank);
        let target = gateways.first().ok_or_else(|| {
            GrpcEndpointError::QpuEndpointNotFound(quantum_processor_id.to_string())
        })?;
        parse_uri(&target.url).map_err(GrpcEndpointError::GrpcError)
    }
}

/// Errors that may occur while trying to resolve a `gRPC` endpoint
#[derive(Debug, thiserror::Error)]
pub enum GrpcEndpointError {
    /// Error due to a bad gRPC configuration
    #[error("Error configuring gRPC request: {0}")]
    GrpcError(#[from] GrpcError<RefreshError>),

    /// Error due to failure to get endpoint for quantum processor
    #[error("Failed to get endpoint for quantum processor: {0}")]
    QpuEndpointRequestFailed(#[from] OpenApiError<GetDefaultEndpointError>),

    /// Error due to failure to get endpoint for quantum processor
    #[error("Failed to get endpoint for the given ID: {0}")]
    EndpointRequestFailed(#[from] OpenApiError<GetEndpointError>),

    /// Error due to failure to get accessors for quantum processor
    #[error("Failed to get accessors for quantum processor: {0}")]
    AccessorRequestFailed(#[from] OpenApiError<ListQuantumProcessorAccessorsError>),

    /// Error due to missing gRPC endpoint for quantum processor
    #[error("Missing gRPC endpoint for quantum processor: {0}")]
    QpuEndpointNotFound(String),

    /// Error due to missing gRPC endpoint for endpoint ID
    #[error("Missing gRPC endpoint for endpoint ID: {0}")]
    EndpointNotFound(String),
}

/// Errors that may occur while trying to use a `gRPC` client
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
