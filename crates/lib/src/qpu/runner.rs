use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};

use enum_as_inner::EnumAsInner;
use num::complex::Complex32;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

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
pub(crate) struct JobId(pub(crate) String);

/// Execute compiled program on a QPU.
pub(crate) async fn submit(
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

pub(crate) async fn retrieve_results(
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

/// Errors that can occur when decoding the results from the QPU
#[derive(Debug, thiserror::Error)]
pub(crate) enum DecodeError {
    #[error("Only 1-dimensional buffer shapes are currently supported")]
    InvalidShape,
    #[error("Expected buffer length {expected}, got {actual}")]
    BufferLength { expected: usize, actual: usize },
}

impl Buffer {
    fn check_shape(&self) -> Result<(), DecodeError> {
        if self.shape.len() == 1 {
            Ok(())
        } else {
            Err(DecodeError::InvalidShape)
        }
    }

    fn assert_len(&self, expected: usize) -> Result<(), DecodeError> {
        let actual = self.data.len();
        if expected == actual {
            Ok(())
        } else {
            Err(DecodeError::BufferLength { expected, actual })
        }
    }
}

/// The raw form of the data which comes back from an execution.
///
/// Generally this should not be used directly, but converted into an appropriate
/// 2-D array.
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Eq)]
struct Buffer {
    shape: Vec<usize>,
    data: ByteBuf,
    dtype: DataType,
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum DataType {
    Float64,
    Int16,
    Complex64,
    Int8,
}

impl Display for DataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Float64 => write!(f, "float64"),
            Self::Int16 => write!(f, "int16"),
            Self::Complex64 => write!(f, "complex64"),
            Self::Int8 => write!(f, "int8"),
        }
    }
}

/// Data from an individual register. Each variant contains a vector with the expected data type
/// where each value in the vector corresponds to a shot.
#[derive(Debug, PartialEq, EnumAsInner)]
pub(crate) enum Register {
    /// Corresponds to the NumPy `float64` type, contains a vector of `f64`.
    F64(Vec<f64>),
    /// Corresponds to the NumPy `int16` type, contains a vector of `i16`.
    I16(Vec<i16>),
    /// Corresponds to the NumPy `complex64` type, contains a vector of [`num::complex::Complex32`].
    Complex32(Vec<Complex32>),
    /// Corresponds to the NumPy `int8` type, contains a vector of `i8`.
    I8(Vec<i8>),
}

#[allow(clippy::cast_possible_wrap)]
impl TryFrom<Buffer> for Register {
    type Error = DecodeError;

    fn try_from(buffer: Buffer) -> Result<Register, Self::Error> {
        const NUM_BYTES_IN_F64: usize = 8;
        const NUM_BYTES_IN_I16: usize = 2;
        const NUM_BYTES_IN_F32: usize = 4;
        const NUM_BYTES_IN_COMPLEX32: usize = NUM_BYTES_IN_F32 * 2;

        buffer.check_shape()?;

        let shots = buffer.shape[0];
        let expected_len: usize = match &buffer.dtype {
            DataType::Float64 => shots * NUM_BYTES_IN_F64,
            DataType::Int16 => shots * NUM_BYTES_IN_I16,
            DataType::Complex64 => shots * NUM_BYTES_IN_COMPLEX32,
            DataType::Int8 => shots,
        };
        buffer.assert_len(expected_len)?;

        Ok(match &buffer.dtype {
            DataType::Float64 => Self::F64(
                buffer
                    .data
                    .chunks_exact(NUM_BYTES_IN_F64)
                    .map(|data| {
                        f64::from_le_bytes(
                            data.try_into()
                                .expect("Length of all the pieces was pre-checked up above!"),
                        )
                    })
                    .collect(),
            ),
            DataType::Int16 => Self::I16(
                buffer
                    .data
                    .chunks_exact(NUM_BYTES_IN_I16)
                    .map(|data| {
                        i16::from_le_bytes(
                            data.try_into()
                                .expect("Length of all the pieces was pre-checked up above!"),
                        )
                    })
                    .collect(),
            ),
            DataType::Complex64 => Self::Complex32(
                buffer
                    .data
                    .chunks_exact(NUM_BYTES_IN_COMPLEX32)
                    .map(|data: &[u8]| {
                        let [real, imaginary]: [f32; 2] =
                            data.chunks_exact(NUM_BYTES_IN_F32)
                                .map(|data| {
                                    f32::from_le_bytes(data.try_into().expect(
                                        "Length of all the pieces was pre-checked up above!",
                                    ))
                                })
                                .collect::<Vec<f32>>()
                                .try_into()
                                .expect("Length of all the pieces was pre-checked up above!");
                        Complex32::new(real, imaginary)
                    })
                    .collect(),
            ),
            DataType::Int8 => Self::I8(
                buffer
                    .data
                    .into_iter()
                    .map(|unsigned: u8| unsigned as i8)
                    .collect(),
            ),
        })
    }
}
