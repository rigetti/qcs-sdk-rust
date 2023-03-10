//! This module provides bindings to for submitting jobs to and retrieving them from
//! Rigetti QPUs using the QCS API.

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

/// The QCS Job ID. Useful for debugging or retrieving results later.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct JobId(pub(crate) String);

impl From<String> for JobId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl ToString for JobId {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

/// Execute compiled program on a QPU.
pub async fn submit(
    quantum_processor_id: &str,
    program: EncryptedControllerJob,
    patch_values: &Parameters,
    client: &Qcs,
) -> Result<JobId, GrpcClientError> {
    let request = ExecuteControllerJobRequest {
        execution_configurations: vec![params_into_job_execution_configuration(patch_values)],
        job: Some(execute_controller_job_request::Job::Encrypted(program)),
        target: Some(execute_controller_job_request::Target::QuantumProcessorId(
            quantum_processor_id.into(),
        )),
    };

    // we expect exactly one job ID since we only submit one execution configuration
    let job_execution_id = client
        .get_controller_client(quantum_processor_id)
        .await?
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
    quantum_processor_id: &str,
    client: &Qcs,
) -> Result<ControllerJobExecutionResult, GrpcClientError> {
    let request = GetControllerJobResultsRequest {
        job_execution_id: Some(job_id.0),
        target: Some(
            get_controller_job_results_request::Target::QuantumProcessorId(
                quantum_processor_id.into(),
            ),
        ),
    };

    client
        .get_controller_client(quantum_processor_id)
        .await?
        .get_controller_job_results(request)
        .await?
        .into_inner()
        .result
        .ok_or_else(|| GrpcClientError::ResponseEmpty("Job Execution Results".into()))
}
