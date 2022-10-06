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

#[derive(Clone)]
pub struct QcsClient {
    config: ClientConfiguration,
    use_gateway: bool,
}

impl QcsClient {
    pub async fn load() -> Result<Self, LoadError> {
        ClientConfiguration::load().await.map(Self::with_config)
    }

    pub fn with_config(config: ClientConfiguration) -> Self {
        Self {
            config,
            use_gateway: true,
        }
    }

    pub fn with_use_gateway(mut self, use_gateway: bool) -> Self {
        self.use_gateway = use_gateway;
        self
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
            .map(|endpoint| {
                dbg!(&endpoint);
                endpoint
            })
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
        if self.use_gateway {
            let accessors = list_quantum_processor_accessors(
                &self.get_openapi_client(),
                quantum_processor_id,
                Some(10),
                None,
            )
            .await?;
            let target = accessors
                .accessors
                .into_iter()
                .find(|acc| {
                    acc.live
                        && acc.priority.unwrap_or(false)
                        && acc.access_type.is_some()
                        && acc.access_type.as_ref().unwrap().as_ref()
                            == &QuantumProcessorAccessorType::GatewayV1
                })
                .ok_or_else(|| GrpcEndpointError::NoEndpoint(quantum_processor_id.into()))?;
            parse_uri(&target.url).map_err(GrpcEndpointError::BadUri)
        } else {
            let default_endpoint =
                get_default_endpoint(&self.get_openapi_client(), quantum_processor_id).await?;
            let addresses = default_endpoint.addresses.as_ref();
            let grpc_address = addresses.grpc.as_ref();

            grpc_address
                .ok_or_else(|| GrpcEndpointError::NoEndpoint(quantum_processor_id.into()))
                .map(|v| parse_uri(v).map_err(GrpcEndpointError::BadUri))?
        }
    }
}

/// Errors that may occur while trying to resolve a gRPC endpoint
#[derive(Debug, thiserror::Error)]
pub enum GrpcEndpointError {
    #[error("Malformed URI for endpoint: {0}")]
    BadUri(#[from] GrpcError),

    #[error("Failed to get endpoint for quantum processor: {0}")]
    RequestFailed(#[from] OpenApiError<GetDefaultEndpointError>),

    #[error("Failed to get accessors for quantum processor: {0}")]
    AccessorRequestFailed(#[from] OpenApiError<ListQuantumProcessorAccessorsError>),

    #[error("Missing gRPC endpoint for quantum processor {0:?}")]
    NoEndpoint(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ClientGrpcError {
    #[error("Failed to resolve the gRPC endoint: {0}")]
    EndpointNotResolved(#[from] GrpcEndpointError),

    #[error("Call failed during gRPC request: {0}")]
    RequestFailed(#[from] Status),

    #[error("Response body had missing data: {0}")]
    ResponseEmpty(String),

    #[error("gRPC error: {0}")]
    GrpcError(#[from] GrpcError),
}

#[derive(Debug, thiserror::Error)]
pub enum ClientOpenApiError<T> {
    #[error("Call failed during http request: {0}")]
    RequestFailed(#[from] OpenApiError<T>),

    #[error("Response value was empty: {0}")]
    ResponseEmpty(String),
}
