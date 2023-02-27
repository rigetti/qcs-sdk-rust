//! This module provides convenience functions to handle compilation,
//! translation, parameter arithmetic rewriting, and results collection.

use std::collections::HashMap;

use num::Complex;
use qcs_api_client_grpc::{
    models::controller::{readout_values, ControllerJobExecutionResult},
    services::controller::{
        get_controller_job_results_request::Target, GetControllerJobResultsRequest,
    },
};

use serde::Serialize;

use crate::qpu::{
    self,
    client::{GrpcClientError, Qcs},
    runner,
};

/// Data from an individual register. Each variant contains a vector with the expected data type
/// where each value in the vector corresponds to a shot.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(untagged)] // Don't include the discriminant name in serialized output.
pub enum Register {
    /// A register of 64-bit floating point numbers
    F64(Vec<f64>),
    /// A register of 16-bit integers
    I16(Vec<i16>),
    /// A register of 32-bit integers
    I32(Vec<i32>),
    /// A register of 64-bit complex numbers
    Complex64(Vec<Complex<f32>>),
    /// A register of 8-bit integers (bytes)
    I8(Vec<i8>),
}

impl From<qpu::runner::Register> for Register {
    fn from(register: qpu::runner::Register) -> Self {
        match register {
            runner::Register::F64(f) => Register::F64(f),
            runner::Register::I16(i) => Register::I16(i),
            runner::Register::Complex32(c) => {
                Register::Complex64(c.iter().map(|c| Complex::<f32>::new(c.re, c.im)).collect())
            }
            runner::Register::I8(i) => Register::I8(i),
        }
    }
}

/// The execution readout data from a particular memory location.
#[derive(Clone, Debug, Serialize)]
pub struct ExecutionResult {
    shape: Vec<usize>,
    data: Register,
    dtype: String,
}

impl From<readout_values::Values> for ExecutionResult {
    fn from(values: readout_values::Values) -> Self {
        match values {
            readout_values::Values::ComplexValues(c) => Self {
                shape: vec![c.values.len(), 1],
                dtype: "complex".into(),
                data: Register::Complex64(
                    c.values
                        .iter()
                        .map(|c| {
                            Complex::<f32>::new(
                                c.real.unwrap_or_default(),
                                c.imaginary.unwrap_or_default(),
                            )
                        })
                        .collect(),
                ),
            },
            readout_values::Values::IntegerValues(i) => Self {
                shape: vec![i.values.len(), 1],
                dtype: "integer".into(),
                data: Register::I32(i.values),
            },
        }
    }
}

/// Execution readout data for all memory locations.
#[derive(Clone, Debug, Serialize)]
pub struct ExecutionResults {
    buffers: HashMap<String, ExecutionResult>,
    execution_duration_microseconds: Option<u64>,
}

impl From<ControllerJobExecutionResult> for ExecutionResults {
    fn from(result: ControllerJobExecutionResult) -> Self {
        let buffers = result
            .readout_values
            .into_iter()
            .filter_map(|(key, value)| {
                value
                    .values
                    .map(ExecutionResult::from)
                    .map(|result| (key, result))
            })
            .collect();

        Self {
            buffers,
            execution_duration_microseconds: result.execution_duration_microseconds,
        }
    }
}

/// Fetches results for the job
///
/// # Errors
///
/// May error if a [`Qcs`] client cannot be constructed, or if the `gRPC`
/// call fails.
pub async fn retrieve_results(
    job_id: &str,
    quantum_processor_id: &str,
    client: &Qcs,
) -> Result<ExecutionResults, GrpcClientError> {
    let request = GetControllerJobResultsRequest {
        job_execution_id: Some(job_id.into()),
        target: Some(Target::QuantumProcessorId(quantum_processor_id.into())),
    };

    client
        .get_controller_client(quantum_processor_id)
        .await?
        .get_controller_job_results(request)
        .await?
        .into_inner()
        .result
        .map(ExecutionResults::from)
        .ok_or_else(|| GrpcClientError::ResponseEmpty("Controller Job Execution Results".into()))
}
