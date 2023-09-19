//! This module provides bindings to for submitting jobs to and retrieving them from
//! Rigetti QPUs using the QCS API.

use std::{fmt, time::Duration};

use cached::proc_macro::cached;
use derive_builder::Builder;
use qcs_api_client_common::{configuration::RefreshError, ClientConfiguration};
pub use qcs_api_client_grpc::channel::Error as GrpcError;
use qcs_api_client_grpc::{
    channel::{parse_uri, wrap_channel_with, RefreshService},
    get_channel_with_timeout,
    models::controller::{
        controller_job_execution_result, data_value::Value, ControllerJobExecutionResult,
        DataValue, EncryptedControllerJob, JobExecutionConfiguration, RealDataValue,
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
        get_default_endpoint as api_get_default_endpoint, get_endpoint, GetDefaultEndpointError,
        GetEndpointError,
    },
    quantum_processors_api::{
        list_quantum_processor_accessors, ListQuantumProcessorAccessorsError,
    },
};
use qcs_api_client_openapi::models::QuantumProcessorAccessorType;
use tonic::transport::Channel;

use crate::executable::Parameters;

use crate::client::{GrpcClientError, Qcs};

const MAX_DECODING_MESSAGE_SIZE_BYTES: usize = 250 * 1024 * 1024;

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
        target: execution_options.get_job_target(quantum_processor_id),
    };

    let mut controller_client = execution_options
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
        target: execution_options.get_results_target(quantum_processor_id),
    };

    let mut controller_client = execution_options
        .get_controller_client(client, quantum_processor_id)
        .await?;

    controller_client
        .get_controller_job_results(request)
        .await
        .map_err(GrpcClientError::RequestFailed)?
        .into_inner()
        .result
        .ok_or_else(|| GrpcClientError::ResponseEmpty("Job Execution Results".into()))
        .map_err(QpuApiError::from)
        .and_then(
            |result| match controller_job_execution_result::Status::from_i32(result.status) {
                Some(controller_job_execution_result::Status::Success) => Ok(result),
                status => Err(QpuApiError::JobExecutionFailed {
                    status: status
                        .map_or("UNDEFINED", |status| status.as_str_name())
                        .to_string(),
                    message: result
                        .status_message
                        .unwrap_or("No message provided.".to_string()),
                }),
            },
        )
}

/// Options available when connecting to a QPU.
///
/// Use [`Default`] to get a reasonable set of defaults, or start with [`QpuConnectionOptionsBuilder`]
/// to build a custom set of options.
// These are aliases because the ExecutionOptions are actually generic over all QPU operations.
pub type QpuConnectionOptions = ExecutionOptions;
/// Builder for setting up [`QpuConnectionOptions`].
pub type QpuConnectionOptionsBuilder = ExecutionOptionsBuilder;

/// Options avaialable when executing a job on a QPU.
///
/// Use [`Default`] to get a reasonable set of defaults, or start with [`ExecutionOptionsBuilder`]
/// to build a custom set of options.
#[derive(Builder, Clone, Debug, Default, PartialEq, Eq)]
pub struct ExecutionOptions {
    #[doc = "The [`ConnectionStrategy`] to use to establish a connection to the QPU."]
    #[builder(default)]
    connection_strategy: ConnectionStrategy,
    #[doc = "The timeout to use for the request, defaults to 30 seconds. If set to `None`, then there is no timeout."]
    #[builder(default = "Some(Duration::from_secs(30))")]
    timeout: Option<Duration>,
}

impl ExecutionOptions {
    /// Get an [`ExecutionOptionsBuilder`] that can be used to build a custom [`ExecutionOptions`].
    #[must_use]
    pub fn builder() -> ExecutionOptionsBuilder {
        ExecutionOptionsBuilder::default()
    }

    /// Get the [`ConnectionStrategy`].
    #[must_use]
    pub fn connection_strategy(&self) -> &ConnectionStrategy {
        &self.connection_strategy
    }

    /// Get the timeout.
    #[must_use]
    pub fn timeout(&self) -> Option<Duration> {
        self.timeout
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

/// Methods that help select and configure a controller service client given a set of
/// [`ExecutionOptions`] and QPU ID.
impl ExecutionOptions {
    fn get_job_target(
        &self,
        quantum_processor_id: Option<&str>,
    ) -> Option<execute_controller_job_request::Target> {
        match self.connection_strategy() {
            ConnectionStrategy::EndpointId(endpoint_id) => Some(
                execute_controller_job_request::Target::EndpointId(endpoint_id.to_string()),
            ),
            ConnectionStrategy::Gateway | ConnectionStrategy::DirectAccess => quantum_processor_id
                .map(String::from)
                .map(execute_controller_job_request::Target::QuantumProcessorId),
        }
    }

    fn get_results_target(
        &self,
        quantum_processor_id: Option<&str>,
    ) -> Option<get_controller_job_results_request::Target> {
        match self.connection_strategy() {
            ConnectionStrategy::EndpointId(endpoint_id) => Some(
                get_controller_job_results_request::Target::EndpointId(endpoint_id.to_string()),
            ),
            ConnectionStrategy::Gateway | ConnectionStrategy::DirectAccess => quantum_processor_id
                .map(String::from)
                .map(get_controller_job_results_request::Target::QuantumProcessorId),
        }
    }

    /// Get a controller client for the given QPU ID.
    pub async fn get_controller_client(
        &self,
        client: &Qcs,
        quantum_processor_id: Option<&str>,
    ) -> Result<ControllerClient<RefreshService<Channel, ClientConfiguration>>, QpuApiError> {
        let service = self
            .get_qpu_grpc_connection(client, quantum_processor_id)
            .await?;
        Ok(ControllerClient::new(service)
            .max_decoding_message_size(MAX_DECODING_MESSAGE_SIZE_BYTES))
    }

    /// Get a GRPC connection to a QPU, without specifying the API to use.
    pub async fn get_qpu_grpc_connection(
        &self,
        client: &Qcs,
        quantum_processor_id: Option<&str>,
    ) -> Result<RefreshService<Channel, ClientConfiguration>, QpuApiError> {
        let address = match self.connection_strategy() {
            ConnectionStrategy::EndpointId(endpoint_id) => {
                let endpoint = get_endpoint(&client.get_openapi_client(), endpoint_id).await?;
                endpoint
                    .addresses
                    .grpc
                    .ok_or_else(|| QpuApiError::EndpointNotFound(endpoint_id.into()))?
            }
            ConnectionStrategy::Gateway => {
                self.get_gateway_address(
                    quantum_processor_id.ok_or(QpuApiError::MissingQpuId)?,
                    client,
                )
                .await?
            }
            ConnectionStrategy::DirectAccess => {
                self.get_default_endpoint_address(
                    quantum_processor_id.ok_or(QpuApiError::MissingQpuId)?,
                    client,
                )
                .await?
            }
        };
        self.grpc_address_to_channel(&address, client)
    }

    fn grpc_address_to_channel(
        &self,
        address: &str,
        client: &Qcs,
    ) -> Result<RefreshService<Channel, ClientConfiguration>, QpuApiError> {
        let uri = parse_uri(address).map_err(QpuApiError::GrpcError)?;
        let channel = get_channel_with_timeout(uri, self.timeout())
            .map_err(|err| QpuApiError::GrpcError(err.into()))?;
        Ok(wrap_channel_with(channel, client.get_config().clone()))
    }

    async fn get_gateway_address(
        &self,
        quantum_processor_id: &str,
        client: &Qcs,
    ) -> Result<String, QpuApiError> {
        get_accessor_with_cache(quantum_processor_id, client).await
    }

    async fn get_default_endpoint_address(
        &self,
        quantum_processor_id: &str,
        client: &Qcs,
    ) -> Result<String, QpuApiError> {
        get_default_endpoint_with_cache(quantum_processor_id, client).await
    }
}

#[cached(
    result = true,
    time = 60,
    time_refresh = true,
    sync_writes = true,
    key = "String",
    convert = r"{ String::from(quantum_processor_id)}"
)]
async fn get_accessor_with_cache(
    quantum_processor_id: &str,
    client: &Qcs,
) -> Result<String, QpuApiError> {
    #[cfg(feature = "tracing")]
    tracing::info!(quantum_processor_id=%quantum_processor_id, "get_accessor cache miss");
    get_accessor(quantum_processor_id, client).await
}

async fn get_accessor(quantum_processor_id: &str, client: &Qcs) -> Result<String, QpuApiError> {
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

#[cached(
    result = true,
    time = 60,
    time_refresh = true,
    sync_writes = true,
    key = "String",
    convert = r"{ String::from(quantum_processor_id)}"
)]
async fn get_default_endpoint_with_cache(
    quantum_processor_id: &str,
    client: &Qcs,
) -> Result<String, QpuApiError> {
    #[cfg(feature = "tracing")]
    tracing::info!(quantum_processor_id=%quantum_processor_id, "get_default_endpoint cache miss");
    get_default_endpoint(quantum_processor_id, client).await
}

async fn get_default_endpoint(
    quantum_processor_id: &str,
    client: &Qcs,
) -> Result<String, QpuApiError> {
    let default_endpoint =
        api_get_default_endpoint(&client.get_openapi_client(), quantum_processor_id).await?;
    let addresses = default_endpoint.addresses.as_ref();
    let grpc_address = addresses.grpc.as_ref();
    grpc_address
        .ok_or_else(|| QpuApiError::QpuEndpointNotFound(quantum_processor_id.into()))
        .cloned()
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

    /// Error that can occur when controller service fails to execute a job
    #[error("The submitted job failed with status: {status}. {message}")]
    JobExecutionFailed {
        /// The status of the failed job.
        status: String,
        /// The message associated with the failed job.
        message: String,
    },
}
