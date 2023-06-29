//! This module provides bindings to for submitting jobs to and retrieving them from
//! Rigetti QPUs using the QCS API.

use std::fmt;

use derive_builder::Builder;
use qcs_api_client_common::{configuration::RefreshError, ClientConfiguration};
pub use qcs_api_client_grpc::channel::Error as GrpcError;
use qcs_api_client_grpc::{
    channel::{parse_uri, wrap_channel_with, RefreshService},
    get_channel,
    models::controller::{
        data_value::Value, ControllerJobExecutionResult, DataValue, EncryptedControllerJob,
        JobExecutionConfiguration, RealDataValue,
    },
    services::controller::{
        controller_client::ControllerClient, execute_controller_job_request,
        get_controller_job_results_request, ExecuteControllerJobRequest,
        GetControllerJobResultsRequest,
    },
};
pub use qcs_api_client_openapi::apis::Error as OpenApiError;
use qcs_api_client_openapi::apis::{
    endpoints_api::{
        get_default_endpoint, get_endpoint, GetDefaultEndpointError, GetEndpointError,
    },
    quantum_processors_api::{
        list_quantum_processor_accessors, ListQuantumProcessorAccessorsError,
    },
};
use qcs_api_client_openapi::models::QuantumProcessorAccessorType;
use tonic::transport::Channel;

use crate::executable::Parameters;

use crate::client::{GrpcClientError, Qcs};

pub(crate) fn params_into_job_execution_configuration(
    params: &Parameters,
) -> JobExecutionConfiguration {
    let memory_values = params
        .iter()
        .map(|(str, value)| {
            (
                str.as_ref().into(),
                DataValue {
                    value: Some(Value::Real(RealDataValue {
                        data: value.clone(),
                    })),
                },
            )
        })
        .collect();

    JobExecutionConfiguration { memory_values }
}

/// The QCS Job ID. Useful for debugging or retrieving results later.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct JobId(pub(crate) String);

impl fmt::Display for JobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <String as fmt::Display>::fmt(&self.0, f)
    }
}

impl From<String> for JobId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// Execute compiled program on a QPU.
///
/// # Arguments
/// * `quantum_processor_id` - The quantum processor to execute the job on. This parameter
///      is required unless using [`ConnectionStrategy::EndpointId`] in `execution_options`
///      to target a specific endpoint ID.
/// * `program` - The compiled program as an [`EncryptedControllerJob`]
/// * `patch_values` - The parameters to use for the execution.
/// * `client` - The [`Qcs`] client to use.
/// * `execution_options` - The [`ExecutionOptions`] to use. If the connection strategy used
///       is [`ConnectionStrategy::EndpointId`] then direct access to that endpoint
///       overrides the `quantum_processor_id` parameter.
pub async fn submit(
    quantum_processor_id: Option<&str>,
    program: EncryptedControllerJob,
    patch_values: &Parameters,
    client: &Qcs,
    execution_options: &ExecutionOptions,
) -> Result<JobId, QpuApiError> {
    #[cfg(feature = "tracing")]
    tracing::debug!(
        "submitting job to {:?} using options {:?}",
        quantum_processor_id,
        execution_options
    );

    let request = ExecuteControllerJobRequest {
        execution_configurations: vec![params_into_job_execution_configuration(patch_values)],
        job: Some(execute_controller_job_request::Job::Encrypted(program)),
        target: Some(
            execution_options
                .connection_strategy
                .get_job_target(quantum_processor_id)?,
        ),
    };

    let mut controller_client = execution_options
        .connection_strategy
        .get_controller_client(client, quantum_processor_id)
        .await?;

    // we expect exactly one job ID since we only submit one execution configuration
    let job_execution_id = controller_client
        .execute_controller_job(request)
        .await
        .map_err(GrpcClientError::RequestFailed)?
        .into_inner()
        .job_execution_ids
        .pop();

    Ok(job_execution_id
        .map(JobId)
        .ok_or_else(|| GrpcClientError::ResponseEmpty("Job Execution ID".into()))?)
}

/// Fetch results from QPU job execution.
///
/// # Arguments
/// * `job_id` - The [`JobId`] to retrieve results for.
/// * `quantum_processor_id` - The quantum processor the job was run on. This parameter
///      is required unless using [`ConnectionStrategy::EndpointId`] in `execution_options`
///      to target a specific endpoint ID.
/// * `client` - The [`Qcs`] client to use.
/// * `execution_options` - The [`ExecutionOptions`] to use. If the connection strategy used
///       is [`ConnectionStrategy::EndpointId`] then direct access to that endpoint
///       overrides the `quantum_processor_id` parameter.
pub async fn retrieve_results(
    job_id: JobId,
    quantum_processor_id: Option<&str>,
    client: &Qcs,
    execution_options: &ExecutionOptions,
) -> Result<ControllerJobExecutionResult, QpuApiError> {
    #[cfg(feature = "tracing")]
    tracing::debug!(
        "retrieving job results for {} on {:?} using options {:?}",
        job_id,
        quantum_processor_id,
        execution_options,
    );

    let request = GetControllerJobResultsRequest {
        job_execution_id: job_id.0,
        target: Some(
            execution_options
                .connection_strategy
                .get_results_target(quantum_processor_id)?,
        ),
    };

    let mut controller_client = execution_options
        .connection_strategy
        .get_controller_client(client, quantum_processor_id)
        .await?;

    Ok(controller_client
        .get_controller_job_results(request)
        .await
        .map_err(GrpcClientError::RequestFailed)?
        .into_inner()
        .result
        .ok_or_else(|| GrpcClientError::ResponseEmpty("Job Execution Results".into()))?)
}

/// Options avaialable when executing a job on a QPU.
///
/// Use [`Default`] to get a reasonable set of defaults, or start with [`ExecutionOptionsBuilder`]
/// to build a custom set of options.
#[derive(Builder, Clone, Debug, Default, PartialEq, Eq)]
pub struct ExecutionOptions {
    #[doc = "The [`ConnectionStrategy`] to use to establish a connection to the QPU."]
    connection_strategy: ConnectionStrategy,
}

impl ExecutionOptions {
    /// Get the [`ConnectionStrategy`]
    #[must_use]
    pub fn connection_strategy(&self) -> &ConnectionStrategy {
        &self.connection_strategy
    }
}

/// The connection strategy to use when submitting and retrieving jobs from a QPU.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ConnectionStrategy {
    /// Connect through the publicly accessible gateway.
    #[default]
    Gateway,
    /// Connect directly to the default endpoint, bypassing the gateway. Should only be used when you
    /// have direct network access and an active reservation.
    DirectAccess,
    /// Connect directly to a specific endpoint using its ID.
    EndpointId(String),
}

/// Methods that help select the right controller service client given a quantum processor ID and
/// [`ConnectionStrategy`].
impl ConnectionStrategy {
    fn get_job_target(
        &self,
        quantum_processor_id: Option<&str>,
    ) -> Result<execute_controller_job_request::Target, QpuApiError> {
        match self {
            Self::EndpointId(endpoint_id) => Ok(
                execute_controller_job_request::Target::EndpointId(endpoint_id.to_string()),
            ),
            Self::Gateway | Self::DirectAccess => {
                Ok(execute_controller_job_request::Target::QuantumProcessorId(
                    quantum_processor_id
                        .ok_or(QpuApiError::MissingQpuId)?
                        .to_string(),
                ))
            }
        }
    }

    fn get_results_target(
        &self,
        quantum_processor_id: Option<&str>,
    ) -> Result<get_controller_job_results_request::Target, QpuApiError> {
        match self {
            Self::EndpointId(endpoint_id) => Ok(
                get_controller_job_results_request::Target::EndpointId(endpoint_id.to_string()),
            ),
            Self::Gateway | Self::DirectAccess => Ok(
                get_controller_job_results_request::Target::QuantumProcessorId(
                    quantum_processor_id
                        .ok_or(QpuApiError::MissingQpuId)?
                        .to_string(),
                ),
            ),
        }
    }

    async fn get_controller_client(
        &self,
        client: &Qcs,
        quantum_processor_id: Option<&str>,
    ) -> Result<ControllerClient<RefreshService<Channel, ClientConfiguration>>, QpuApiError> {
        let address = match self {
            Self::EndpointId(endpoint_id) => {
                let endpoint = get_endpoint(&client.get_openapi_client(), endpoint_id).await?;
                endpoint
                    .addresses
                    .grpc
                    .ok_or_else(|| QpuApiError::EndpointNotFound(endpoint_id.into()))?
            }
            Self::Gateway => {
                self.get_gateway_address(
                    quantum_processor_id.ok_or(QpuApiError::MissingQpuId)?,
                    client,
                )
                .await?
            }
            Self::DirectAccess => {
                self.get_default_endpoint_address(
                    quantum_processor_id.ok_or(QpuApiError::MissingQpuId)?,
                    client,
                )
                .await?
            }
        };
        Self::grpc_address_to_client(&address, client)
    }

    fn grpc_address_to_client(
        address: &str,
        client: &Qcs,
    ) -> Result<ControllerClient<RefreshService<Channel, ClientConfiguration>>, QpuApiError> {
        let uri = parse_uri(address).map_err(QpuApiError::GrpcError)?;
        let channel = get_channel(uri).map_err(|err| QpuApiError::GrpcError(err.into()))?;
        let service = wrap_channel_with(channel, client.get_config().clone());
        Ok(ControllerClient::new(service))
    }

    async fn get_gateway_address(
        &self,
        quantum_processor_id: &str,
        client: &Qcs,
    ) -> Result<String, QpuApiError> {
        let mut min = None;
        let mut next_page_token = None;
        loop {
            let accessors = list_quantum_processor_accessors(
                &client.get_openapi_client(),
                quantum_processor_id,
                Some(100),
                next_page_token.as_deref(),
            )
            .await?;

            let accessor = accessors
                .accessors
                .into_iter()
                .filter(|acc| {
                    acc.live
                    // `as_deref` needed to work around the `Option<Box<_>>` type.
                    && acc.access_type.as_deref() == Some(&QuantumProcessorAccessorType::GatewayV1)
                })
                .min_by_key(|acc| acc.rank.unwrap_or(i64::MAX));

            min = std::cmp::min_by_key(min, accessor, |acc| {
                acc.as_ref().and_then(|acc| acc.rank).unwrap_or(i64::MAX)
            });

            next_page_token = accessors.next_page_token.clone();
            if next_page_token.is_none() {
                break;
            }
        }
        min.map(|accessor| accessor.url)
            .ok_or_else(|| QpuApiError::GatewayNotFound(quantum_processor_id.to_string()))
    }

    async fn get_default_endpoint_address(
        &self,
        quantum_processor_id: &str,
        client: &Qcs,
    ) -> Result<String, QpuApiError> {
        let default_endpoint =
            get_default_endpoint(&client.get_openapi_client(), quantum_processor_id).await?;
        let addresses = default_endpoint.addresses.as_ref();
        let grpc_address = addresses.grpc.as_ref();
        grpc_address
            .ok_or_else(|| QpuApiError::QpuEndpointNotFound(quantum_processor_id.into()))
            .cloned()
    }
}

/// Errors that can occur while attempting to establish a connection to the QPU.
#[derive(Debug, thiserror::Error)]
pub enum QpuApiError {
    /// Error due to a bad gRPC configuration
    #[error("Error configuring gRPC request: {0}")]
    GrpcError(#[from] GrpcError<RefreshError>),

    /// Error due to missing gRPC endpoint for endpoint ID
    #[error("Missing gRPC endpoint for endpoint ID: {0}")]
    EndpointNotFound(String),

    /// Error due to missing gRPC endpoint for quantum processor
    #[error("Missing gRPC endpoint for quantum processor: {0}")]
    QpuEndpointNotFound(String),

    /// Error due to failure to get endpoint for quantum processor
    #[error("Failed to get endpoint for quantum processor: {0}")]
    QpuEndpointRequestFailed(#[from] OpenApiError<GetDefaultEndpointError>),

    /// Error due to failure to get accessors for quantum processor
    #[error("Failed to get accessors for quantum processor: {0}")]
    AccessorRequestFailed(#[from] OpenApiError<ListQuantumProcessorAccessorsError>),

    /// Error due to failure to find gateway for quantum processor
    #[error("No gateway found for quantum processor: {0}")]
    GatewayNotFound(String),

    /// Error due to failure to get endpoint for quantum processor
    #[error("Failed to get endpoint for the given ID: {0}")]
    EndpointRequestFailed(#[from] OpenApiError<GetEndpointError>),

    /// Errors that may occur while trying to use a `gRPC` client
    #[error(transparent)]
    GrpcClientError(#[from] GrpcClientError),

    /// Error due to missing quantum processor ID and endpoint ID.
    #[error("A quantum processor ID must be provided if not connecting directly to an endpoint ID with ConnectionStrategy::EndpointId")]
    MissingQpuId,
}
