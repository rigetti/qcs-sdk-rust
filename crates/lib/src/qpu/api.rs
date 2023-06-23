//! This module provides bindings to for submitting jobs to and retrieving them from
//! Rigetti QPUs using the QCS API.

use std::fmt;

use qcs_api_client_common::ClientConfiguration;
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
use qcs_api_client_openapi::apis::{
    endpoints_api::{get_default_endpoint, get_endpoint},
    quantum_processors_api::list_quantum_processor_accessors,
};
use qcs_api_client_openapi::models::QuantumProcessorAccessorType;
use tonic::transport::Channel;

use crate::{client::GrpcEndpointError, executable::Parameters};

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

/// A QCS Job execution target, either a Quantum Processor ID or a specific endpoint.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum JobTarget {
    /// Execute against a QPU's default endpoint.
    QuantumProcessorId(String),

    /// Execute against a specific endpoint by ID.
    EndpointId(String),
}

impl fmt::Display for JobTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EndpointId(id) | Self::QuantumProcessorId(id) => write!(f, "{id}"),
        }
    }
}

impl From<&JobTarget> for execute_controller_job_request::Target {
    fn from(value: &JobTarget) -> Self {
        match value {
            JobTarget::EndpointId(v) => Self::EndpointId(v.into()),
            JobTarget::QuantumProcessorId(v) => Self::QuantumProcessorId(v.into()),
        }
    }
}

impl From<&JobTarget> for get_controller_job_results_request::Target {
    fn from(value: &JobTarget) -> Self {
        match value {
            JobTarget::EndpointId(v) => Self::EndpointId(v.into()),
            JobTarget::QuantumProcessorId(v) => Self::QuantumProcessorId(v.into()),
        }
    }
}

impl JobTarget {
    async fn get_controller_client(
        &self,
        client: &Qcs,
        connection_strategy: ConnectionStrategy,
    ) -> Result<ControllerClient<RefreshService<Channel, ClientConfiguration>>, GrpcEndpointError>
    {
        match self {
            Self::EndpointId(endpoint_id) => {
                let endpoint = get_endpoint(&client.get_openapi_client(), endpoint_id).await?;
                let grpc_address = endpoint
                    .addresses
                    .grpc
                    .ok_or_else(|| GrpcEndpointError::EndpointNotFound(endpoint_id.into()))?;
                Self::grpc_address_to_client(&grpc_address, client)
            }
            Self::QuantumProcessorId(quantum_processor_id) => {
                let address = match connection_strategy {
                    ConnectionStrategy::GatewayAlways => {
                        self.get_gateway_address(quantum_processor_id, client)
                            .await?
                    }
                    ConnectionStrategy::DirectAccessAlways => {
                        self.get_default_endpoint_address(quantum_processor_id, client)
                            .await?
                    }
                };
                Self::grpc_address_to_client(&address, client)
            }
        }
    }

    fn grpc_address_to_client(
        address: &str,
        client: &Qcs,
    ) -> Result<ControllerClient<RefreshService<Channel, ClientConfiguration>>, GrpcEndpointError>
    {
        let uri = parse_uri(address).map_err(GrpcEndpointError::GrpcError)?;
        let channel = get_channel(uri).map_err(|err| GrpcEndpointError::GrpcError(err.into()))?;
        let service = wrap_channel_with(channel, client.get_config().clone());
        Ok(ControllerClient::new(service))
    }

    async fn get_gateway_address(
        &self,
        quantum_processor_id: &str,
        client: &Qcs,
    ) -> Result<String, GrpcEndpointError> {
        let mut gateways = Vec::new();
        let mut next_page_token = None;
        loop {
            let accessors = list_quantum_processor_accessors(
                &client.get_openapi_client(),
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
        gateways
            .first()
            .map(|accessor| accessor.url.clone())
            .ok_or_else(|| GrpcEndpointError::QpuEndpointNotFound(quantum_processor_id.to_string()))
    }

    async fn get_default_endpoint_address(
        &self,
        quantum_processor_id: &str,
        client: &Qcs,
    ) -> Result<String, GrpcEndpointError> {
        let default_endpoint =
            get_default_endpoint(&client.get_openapi_client(), quantum_processor_id).await?;
        let addresses = default_endpoint.addresses.as_ref();
        let grpc_address = addresses.grpc.as_ref();
        grpc_address
            .ok_or_else(|| GrpcEndpointError::QpuEndpointNotFound(quantum_processor_id.into()))
            .cloned()
    }
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
/// * `job_target` - A [`JobTarget`] describing the QPU or endpoint to run the program on.
/// * `program` - The compiled program as an [`EncryptedControllerJob`]
/// * `patch_values` - The parameters to use for the execution.
/// * `client` - The [`Qcs`] client to use.
/// * `connection_strategy` - The [`ConnectionStrategy`] to use. If `job_target` is an
///       endpoint ID, then direct access to that endpoint ID overrides this parameter.
pub async fn submit(
    job_target: &JobTarget,
    program: EncryptedControllerJob,
    patch_values: &Parameters,
    client: &Qcs,
    connection_strategy: ConnectionStrategy,
) -> Result<JobId, GrpcClientError> {
    #[cfg(feature = "tracing")]
    tracing::debug!("submitting job to {}", job_target);

    let request = ExecuteControllerJobRequest {
        execution_configurations: vec![params_into_job_execution_configuration(patch_values)],
        job: Some(execute_controller_job_request::Job::Encrypted(program)),
        target: Some(job_target.into()),
    };

    let mut controller_client = job_target
        .get_controller_client(client, connection_strategy)
        .await?;

    // we expect exactly one job ID since we only submit one execution configuration
    let job_execution_id = controller_client
        .execute_controller_job(request)
        .await?
        .into_inner()
        .job_execution_ids
        .pop();

    job_execution_id
        .map(JobId)
        .ok_or_else(|| GrpcClientError::ResponseEmpty("Job Execution ID".into()))
}

/// Fetch results from QPU job execution.
///
/// # Arguments
/// * `job_id` - The [`JobId`] to retrieve results for.
/// * `job_target` - The [`JobTarget`] the job was run on.
/// * `client` - The [`Qcs`] client to use.
/// * `connection_strategy` - The [`ConnectionStrategy`] to use. If `job_target` is an
///       endpoint ID, then direct access to that endpoint ID overrides this parameter.
pub async fn retrieve_results(
    job_id: JobId,
    job_target: &JobTarget,
    client: &Qcs,
    connection_strategy: ConnectionStrategy,
) -> Result<ControllerJobExecutionResult, GrpcClientError> {
    #[cfg(feature = "tracing")]
    tracing::debug!("retrieving job results for {} on {}", job_id, job_target,);

    let request = GetControllerJobResultsRequest {
        job_execution_id: job_id.0,
        target: Some(job_target.into()),
    };

    let mut controller_client = job_target
        .get_controller_client(client, connection_strategy)
        .await?;

    controller_client
        .get_controller_job_results(request)
        .await?
        .into_inner()
        .result
        .ok_or_else(|| GrpcClientError::ResponseEmpty("Job Execution Results".into()))
}

/// The connection strategy to use when submitting and retrieving jobs from a QPU.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ConnectionStrategy {
    /// Connect through the publicly accessible gateway.
    #[default]
    GatewayAlways,
    /// Connect directly to the QPU endpoint, bypassing the gateway. Should only be used when you
    /// have direct access and an active reservation.
    DirectAccessAlways,
}
