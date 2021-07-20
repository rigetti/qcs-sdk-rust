use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};

use eyre::{eyre, Result, WrapErr};
use num::complex::Complex32;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

use qcs_api::models::EngagementWithCredentials;
use rpcq::{Client, Credentials, RPCRequest};

pub(crate) fn execute(
    program: String,
    engagement: EngagementWithCredentials,
) -> Result<HashMap<String, Buffer>> {
    let params = QPUParams::from_program(program);
    let EngagementWithCredentials {
        address,
        credentials,
        ..
    } = engagement;

    // This is a hack to allow testing without credentials since ZAP is absurd
    let client = if credentials.server_public.is_empty() {
        Client::new(&address).wrap_err("Unable to connect to the QPU (Lodgepole)")?
    } else {
        let credentials = Credentials {
            client_secret_key: credentials.client_secret,
            client_public_key: credentials.client_public,
            server_public_key: credentials.server_public,
        };
        Client::new_with_credentials(&address, &credentials)
            .wrap_err("Unable to connect to the QPU (Lodgepole)")?
    };

    // let client = Client::new(&address).unwrap();
    let job_id: String = client
        .run_request(&params.into())
        .wrap_err("While attempting to send the program to the QPU (Lodgepole)")?;
    let get_buffers_request = GetBuffersRequest::new(job_id);
    client
        .run_request(&get_buffers_request.into())
        .wrap_err("While attempting to receive results from to the QPU (Lodgepole)")
}

#[derive(Serialize, Debug)]
struct QPUParams {
    request: QPURequest,
    priority: u8,
}

impl QPUParams {
    fn from_program(program: String) -> Self {
        Self {
            request: QPURequest::from_program(program),
            priority: 1,
        }
    }
}

impl From<QPUParams> for RPCRequest<QPUParams> {
    fn from(params: QPUParams) -> Self {
        RPCRequest::new("execute_qpu_request", params)
    }
}

#[derive(Serialize, Debug)]
#[serde(tag = "_type")]
struct QPURequest {
    id: String,
    program: String,
    patch_values: HashMap<String, Vec<f64>>,
}

impl QPURequest {
    fn from_program(program: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            program,
            // TODO: Read `pyquil.api._qpu.QPU._build_patch_values` and implement arithmetic
            patch_values: HashMap::new(),
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

impl From<GetBuffersRequest> for RPCRequest<GetBuffersRequest> {
    fn from(req: GetBuffersRequest) -> Self {
        RPCRequest::new("get_buffers", req)
    }
}

/// The raw form of the data which comes back from an execution.
///
/// Generally this should not be used directly, but converted into an appropriate
/// 2-D array.
#[derive(Deserialize, Debug, PartialEq)]
pub struct Buffer {
    shape: Vec<usize>,
    data: ByteBuf,
    dtype: DataType,
}

#[derive(Deserialize, Debug, PartialEq)]
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

/// The types of data for an individual register that can come back from a QPU.
#[derive(Debug, PartialEq)]
pub enum QPUResult {
    F64(Vec<f64>),
    I16(Vec<i16>),
    Complex32(Vec<Complex32>),
    I8(Vec<i8>),
}

#[allow(clippy::cast_possible_wrap)]
impl TryFrom<Buffer> for QPUResult {
    type Error = eyre::Error;

    fn try_from(buffer: Buffer) -> Result<QPUResult, Self::Error> {
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

    use rpcq::RPCResponse;

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
        let expected = QPUResult::I16(vec![1, 2, 3]);

        let actual: QPUResult = buf.try_into().expect("Failed to convert into u16s");

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
        let expected = QPUResult::I8(vec![-1, 0, 1]);

        let actual: QPUResult = buf.try_into().expect("Failed to convert into u16s");

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
        let expected = QPUResult::F64(vec![1.0, 2.0, 3.0]);

        let actual: QPUResult = buf.try_into().expect("Failed to convert into f64s");

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
        let expected = QPUResult::Complex32(vec![
            Complex32::new(1.0, 0.0),
            Complex32::new(2.0, 0.0),
            Complex32::new(3.0, 0.0),
        ]);

        let actual: QPUResult = buf.try_into().expect("Failed to convert into Complex32s");

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

        let q0_data: QPUResult = q0.try_into().unwrap();
        let q1_data: QPUResult = q1.try_into().unwrap();
        let expected_data = QPUResult::I8(vec![0, 1]);
        assert_eq!(q0_data, expected_data);
        assert_eq!(q1_data, expected_data);
    }
}
