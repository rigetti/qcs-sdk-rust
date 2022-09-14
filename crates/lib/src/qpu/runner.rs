//! Interface to the QPU's API.

use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};

use enum_as_inner::EnumAsInner;
use num::complex::Complex32;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

use crate::executable::Parameters;

use super::engagement;
use super::rpcq::{Client, Error as RPCQError, RPCRequest};

/// The QCS Job ID. Useful for debugging or retrieving results later.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct JobId(pub(crate) String);

/// Execute compiled program on a QPU.
pub(crate) fn submit(
    program: &str,
    patch_values: &Parameters,
    client: &Client,
) -> Result<JobId, Error> {
    let params = QPUParams::from_program(program, patch_values);
    let request = RPCRequest::from(&params);
    let job_id: String = client.run_request(&request)?;
    Ok(JobId(job_id))
}

pub(crate) fn retrieve_results(
    job_id: JobId,
    client: &Client,
) -> Result<GetExecutionResultsResponse, Error> {
    let get_buffers_request = GetExecutionResultsRequest::new(job_id.0);
    client
        .run_request(&RPCRequest::from(&get_buffers_request))
        .map_err(Error::from)
}

/// All of the possible errors for this module
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error connecting to the QPU")]
    Connection(#[source] RPCQError),
    #[error("Error getting engagement for QPU")]
    Engagement(#[source] engagement::Error),
    #[error("An error not expected to occur—if encountered it may indicate bug in this library")]
    Unexpected(#[source] RPCQError),
    #[error("An error was returned from the QPU: {0}")]
    Qpu(String),
    #[error("An error was encountered when claiming the client lock: {0}")]
    ClientLock(String),
}

impl From<RPCQError> for Error {
    fn from(err: RPCQError) -> Self {
        match err {
            RPCQError::SocketCreation(_)
            | RPCQError::Communication(_)
            | RPCQError::ResponseIdMismatch => Self::Connection(err),
            RPCQError::AuthSetup(_)
            | RPCQError::Serialization(_)
            | RPCQError::Deserialization(_) => Self::Unexpected(err),
            RPCQError::Response(message) => Self::Qpu(message),
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
struct QPUParams<'request> {
    request: QPURequest<'request>,
    priority: u8,
}

impl<'request> QPUParams<'request> {
    fn from_program(program: &'request str, patch_values: &'request Parameters) -> Self {
        Self {
            request: QPURequest::from_program(program, patch_values),
            priority: 1,
        }
    }
}

impl<'params> From<&'params QPUParams<'params>> for RPCRequest<'params, QPUParams<'params>> {
    fn from(params: &'params QPUParams) -> Self {
        RPCRequest::new("execute_qpu_request", params)
    }
}

#[derive(Serialize, Debug, PartialEq, Clone)]
#[serde(tag = "_type")]
struct QPURequest<'request> {
    id: String,
    program: &'request str,
    patch_values: &'request Parameters,
}

impl<'request> QPURequest<'request> {
    fn from_program(program: &'request str, patch_values: &'request Parameters) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            program,
            patch_values,
        }
    }
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq, Hash)]
struct GetExecutionResultsRequest {
    job_id: String,
    wait: bool,
}

impl GetExecutionResultsRequest {
    fn new(job_id: String) -> Self {
        Self { job_id, wait: true }
    }
}

impl<'request> From<&'request GetExecutionResultsRequest>
    for RPCRequest<'request, GetExecutionResultsRequest>
{
    fn from(req: &'request GetExecutionResultsRequest) -> Self {
        RPCRequest::new("get_execution_results", req)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct GetExecutionResultsResponse {
    pub buffers: HashMap<String, Buffer>,
    #[serde(default)]
    pub execution_duration_microseconds: Option<u64>,
}

/// The raw form of the data which comes back from an execution.
///
/// Generally this should not be used directly, but converted into an appropriate
/// 2-D array.
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Eq)]
pub struct Buffer {
    pub shape: Vec<usize>,
    pub data: ByteBuf,
    pub dtype: DataType,
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
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

/// Errors that can occur when decoding the results from the QPU
#[derive(Debug, thiserror::Error)]
pub(crate) enum DecodeError {
    #[error("Only 1-dimensional buffer shapes are currently supported")]
    InvalidShape,
    #[error("Expected buffer length {expected}, got {actual}")]
    BufferLength { expected: usize, actual: usize },
    #[error("Missing expected buffer named {0}, did you forget to MEASURE?")]
    MissingBuffer(String),
    #[error("This SDK expects contiguous memory, but {register}[{index}] was missing.")]
    ContiguousMemory { register: String, index: usize },
    #[error("A single register should have the same type for all shots, got mixed types.")]
    MixedTypes,
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

#[cfg(test)]
mod describe_buffer {
    use std::{collections::HashMap, convert::TryInto};

    use num::complex::Complex32;
    use serde_bytes::ByteBuf;

    use crate::qpu::{
        rpcq::RPCResponse,
        runner::{Buffer, DataType},
        Register,
    };

    #[test]
    fn it_converts_numpy_int16() {
        // Data extracted from Python's NumPy
        let data = [1, 0, 2, 0, 3, 0];
        let buf = Buffer {
            shape: vec![3],
            data: ByteBuf::from(data),
            dtype: DataType::Int16,
        };
        let expected = Register::I16(vec![1, 2, 3]);

        let actual: Register = buf.try_into().expect("Failed to convert into u16s");

        assert_eq!(expected, actual);
    }

    #[test]
    fn it_converts_numpy_int8() {
        // Data extracted from Python's NumPy
        let data = [255, 0, 1];
        let buf = Buffer {
            shape: vec![3],
            data: ByteBuf::from(data),
            dtype: DataType::Int8,
        };
        let expected = Register::I8(vec![-1, 0, 1]);

        let actual: Register = buf.try_into().expect("Failed to convert into u16s");

        assert_eq!(expected, actual);
    }

    #[test]
    fn it_converts_numpy_float64() {
        // Data extracted from Python's NumPy
        let data = [
            0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 8, 64,
        ];
        let buf = Buffer {
            shape: vec![3],
            data: ByteBuf::from(data),
            dtype: DataType::Float64,
        };
        let expected = Register::F64(vec![1.0, 2.0, 3.0]);

        let actual: Register = buf.try_into().expect("Failed to convert into f64s");

        assert_eq!(expected, actual);
    }

    #[test]
    fn it_converts_numpy_complex64() {
        // Data extracted from Python's NumPy
        let data = [
            0, 0, 128, 63, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 64, 64, 0, 0, 0, 0,
        ];
        let buf = Buffer {
            shape: vec![3],
            data: ByteBuf::from(data),
            dtype: DataType::Complex64,
        };
        let expected = Register::Complex32(vec![
            Complex32::new(1.0, 0.0),
            Complex32::new(2.0, 0.0),
            Complex32::new(3.0, 0.0),
        ]);

        let actual: Register = buf.try_into().expect("Failed to convert into Complex32s");

        assert_eq!(expected, actual);
    }

    #[test]
    fn it_deserializes_properly() {
        // Real response data for a Bell State program retrieved from Aspen 9
        let hex = std::fs::read_to_string("tests/bell_state_response_data.hex")
            .expect("Unable to load Aspen-9 Bell State raw response");
        let data = hex::decode(hex).unwrap();
        let resp: RPCResponse<HashMap<String, Buffer>> =
            rmp_serde::from_read(data.as_slice()).unwrap();
        let mut buffers = match resp {
            RPCResponse::RPCReply { result, .. } => result,
            RPCResponse::RPCError { .. } => unreachable!(),
        };
        let q0 = buffers.remove("q0").expect("Could not find buffer q0");
        let q1 = buffers.remove("q1").expect("Could not find buffer q1");

        assert_eq!(q0, q1);
        assert!(matches!(q0.dtype, DataType::Int8));

        let q0_data: Register = q0.try_into().unwrap();
        let q1_data: Register = q1.try_into().unwrap();
        let expected_data = Register::I8(vec![0, 1]);
        assert_eq!(q0_data, expected_data);
        assert_eq!(q1_data, expected_data);
    }
}
