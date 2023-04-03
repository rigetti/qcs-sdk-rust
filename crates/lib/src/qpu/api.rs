//! This module provides bindings to for submitting jobs to and retrieving them from
//! Rigetti QPUs using the QCS API.

use std::fmt;

use qcs_api_client_grpc::{
    models::controller::{
        data_value::Value, ControllerJobExecutionResult, DataValue, EncryptedControllerJob,
        JobExecutionConfiguration, RealDataValue,
    },
    services::controller::{
        execute_controller_job_request, get_controller_job_results_request,
        ExecuteControllerJobRequest, GetControllerJobResultsRequest,
    },
};

use crate::executable::Parameters;

use super::client::{GrpcClientError, Qcs};

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
pub async fn submit(
    job_target: &JobTarget,
    program: EncryptedControllerJob,
    patch_values: &Parameters,
    client: &Qcs,
) -> Result<JobId, GrpcClientError> {
    #[cfg(feature = "tracing")]
    tracing::debug!("submitting job to {}", job_target);

    let request = ExecuteControllerJobRequest {
        execution_configurations: vec![params_into_job_execution_configuration(patch_values)],
        job: Some(execute_controller_job_request::Job::Encrypted(program)),
        target: Some(job_target.into()),
    };

    let mut controller_client = match job_target {
        JobTarget::EndpointId(endpoint_id) => {
            client
                .get_controller_client_with_endpoint_id(endpoint_id)
                .await
        }
        JobTarget::QuantumProcessorId(quantum_processor_id) => {
            client.get_controller_client(quantum_processor_id).await
        }
    }?;

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
pub async fn retrieve_results(
    job_id: JobId,
    job_target: &JobTarget,
    client: &Qcs,
) -> Result<ControllerJobExecutionResult, GrpcClientError> {
    #[cfg(feature = "tracing")]
    tracing::debug!("retrieving job results for {} on {}", job_id, job_target,);

    let request = GetControllerJobResultsRequest {
        job_execution_id: Some(job_id.0),
        target: Some(job_target.into()),
    };

    let mut controller_client = match job_target {
        JobTarget::EndpointId(endpoint_id) => {
            client
                .get_controller_client_with_endpoint_id(endpoint_id)
                .await
        }
        JobTarget::QuantumProcessorId(quantum_processor_id) => {
            client.get_controller_client(quantum_processor_id).await
        }
    }?;

    controller_client
        .get_controller_job_results(request)
        .await?
        .into_inner()
        .result
        .ok_or_else(|| GrpcClientError::ResponseEmpty("Job Execution Results".into()))
}
