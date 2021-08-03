use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};

use enum_as_inner::EnumAsInner;
use eyre::{eyre, Result, WrapErr};
use log::{debug, trace, warn};
use num::complex::Complex32;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

use qcs_api::models::EngagementWithCredentials;

use crate::executable::Parameters;

use super::rpcq::{Client, Credentials, RPCRequest};

pub(crate) fn execute(
    program: &str,
    engagement: &EngagementWithCredentials,
    patch_values: &Parameters,
) -> Result<HashMap<String, Buffer>> {
    let params = QPUParams::from_program(program, patch_values);
    let EngagementWithCredentials {
        address,
        credentials,
        ..
    } = engagement;

    // This is a hack to allow testing without credentials since ZAP is absurd
    let client = if credentials.server_public.is_empty() {
        warn!(
            "Connecting to Lodgepole on {} with no credentials.",
            address
        );
        Client::new(address).wrap_err("Unable to connect to the QPU (Lodgepole)")?
    } else {
        let credentials = Credentials {
            client_secret_key: &credentials.client_secret,
            client_public_key: &credentials.client_public,
            server_public_key: &credentials.server_public,
        };
        trace!("Connecting to Lodgepole at {} with credentials", &address);
        Client::new_with_credentials(address, &credentials)
            .wrap_err("Unable to connect to the QPU (Lodgepole)")?
    };

    let request = RPCRequest::from(&params);
    let job_id: String = client
        .run_request(&request)
        .wrap_err("While attempting to send the program to the QPU (Lodgepole)")?;
    debug!("Received job ID {} from Lodgepole", &job_id);
    let get_buffers_request = GetBuffersRequest::new(job_id);
    client
        .run_request(&RPCRequest::from(&get_buffers_request))
        .wrap_err("While attempting to receive results from to the QPU (Lodgepole)")
}

#[derive(Serialize, Debug)]
struct QPUParams<'request> {
    request: QPURequest<'request>,
    priority: u8,
}

impl<'request> QPUParams<'request> {
    fn from_program(program: &'request str, patch_values: PatchValues<'request>) -> Self {
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

type PatchValues<'request> = &'request HashMap<&'request str, Vec<f64>>;

#[derive(Serialize, Debug)]
#[serde(tag = "_type")]
struct QPURequest<'request> {
    id: String,
    program: &'request str,
    patch_values: PatchValues<'request>,
}

impl<'request> QPURequest<'request> {
    fn from_program(program: &'request str, patch_values: PatchValues<'request>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            program,
            patch_values,
        }
    }
}

#[derive(Serialize, Debug)]
struct GetBuffersRequest {
    job_id: String,
    wait: bool,
}

impl GetBuffersRequest {
    fn new(job_id: String) -> Self {
        Self { job_id, wait: true }
    }
}

impl<'request> From<&'request GetBuffersRequest> for RPCRequest<'request, GetBuffersRequest> {
    fn from(req: &'request GetBuffersRequest) -> Self {
        RPCRequest::new("get_buffers", req)
    }
}

/// The raw form of the data which comes back from an execution.
///
/// Generally this should not be used directly, but converted into an appropriate
/// 2-D array.
#[derive(Deserialize, Debug, PartialEq)]
pub(crate) struct Buffer {
    shape: Vec<usize>,
    data: ByteBuf,
    dtype: DataType,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum DataType {
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

impl Buffer {
    fn check_shape(&self) -> eyre::Result<()> {
        if self.shape.len() == 1 {
            Ok(())
        } else {
            Err(eyre!(
                "Only 1-dimensional buffer shapes are currently supported"
            ))
        }
    }

    fn assert_len(&self, expected: usize) -> eyre::Result<()> {
        let actual = self.data.len();
        if expected == actual {
            Ok(())
        } else {
            Err(eyre!("Expected buffer length {}, got {}", expected, actual,))
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
    /// Corresponds to the NumPy `complex64` type, contains a vector of [`num::Complex32`].
    Complex32(Vec<Complex32>),
    /// Corresponds to the NumPy `int8` type, contains a vector of `i8`.
    I8(Vec<i8>),
}

#[allow(clippy::cast_possible_wrap)]
impl TryFrom<Buffer> for Register {
    type Error = eyre::Error;

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
    use std::convert::TryInto;

    use crate::qpu::rpcq::RPCResponse;

    use super::*;

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
        let hex = std::fs::read_to_string("tests/bell_state.hex")
            .expect("Unable to load Aspen-9 Bell State raw response");
        let data = hex::decode(hex).unwrap();
        let resp: RPCResponse<HashMap<String, Buffer>> =
            rmp_serde::from_read(data.as_slice()).unwrap();
        let mut buffers = match resp {
            RPCResponse::RPCReply { result, .. } => result,
            _ => unreachable!(),
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
