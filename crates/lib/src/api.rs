use log::warn;
use qcs_api::models::TranslateNativeQuilToEncryptedBinaryResponse;
use serde::Serialize;

use crate::{
    configuration::Configuration,
    qpu::{
        self, engagement, quilc,
        rpcq::Client,
        runner::{self, JobId},
        translation,
    },
};
use std::{collections::HashMap, convert::TryFrom, sync::Mutex};

/// Uses quilc to convert a Quil program to native Quil
pub async fn compile(
    quil: &str,
    quantum_processor_id: &str,
    config: &Configuration,
) -> Result<String, Box<dyn std::error::Error>> {
    let isa = qpu::get_isa(quantum_processor_id, config)
        .await
        .map_err(|e| e.to_string())?;
    quilc::compile_program(quil, isa, config)
        .map_err(|e| e.into())
        .map(|p| p.0)
}

/// Translates a native Quil program into an executable
pub async fn translate(
    native_quil: &str,
    shots: u16,
    quantum_processor_id: &str,
    config: &Configuration,
) -> Result<TranslateNativeQuilToEncryptedBinaryResponse, translation::Error> {
    translation::translate(
        qpu::RewrittenQuil(native_quil.to_string()),
        shots,
        quantum_processor_id,
        config,
    )
    .await
}

/// Submits an executable `program` to be run on the specified QPU
pub async fn submit(
    program: &str,
    patch_values: HashMap<Box<str>, Vec<f64>>,
    quantum_processor_id: &str,
    config: &Configuration,
) -> Result<String, runner::Error> {
    let engagement = engagement::get(String::from(quantum_processor_id), config)
        .await
        .map_err(|e| runner::Error::Qpu(format!("Unable to get engagement: {:?}", e)))?;
    let rpcq_client = Client::try_from(&engagement)
        .map_err(|e| {
            warn!("Unable to connect to QPU via RPCQ: {:?}", e);
            runner::Error::Qpu(format!("Unable to connect to QPU via RPCQ: {:?}", e))
        })
        .map(Mutex::new)?;

    let client = rpcq_client.lock().unwrap();
    let job_id = runner::submit(program, &patch_values, &client)?;

    Ok(job_id.0)
}

/// Data from an individual register. Each variant contains a vector with the expected data type
/// where each value in the vector corresponds to a shot.
#[derive(Debug, PartialEq, Serialize)]
#[serde(untagged)] // Don't include the discriminant name in serialized output.
pub enum Register {
    F64(Vec<f64>),
    I16(Vec<i16>),
    Complex64(Vec<f32>),
    I8(Vec<i8>),
}

impl From<qpu::runner::Register> for Register {
    fn from(register: qpu::runner::Register) -> Self {
        match register {
            runner::Register::F64(f) => Register::F64(f),
            runner::Register::I16(i) => Register::I16(i),
            runner::Register::Complex32(c) => {
                Register::Complex64(c.iter().flat_map(|c| vec![c.re, c.im]).collect())
            }
            runner::Register::I8(i) => Register::I8(i),
        }
    }
}

#[derive(Serialize)]
pub struct ExecutionResult {
    shape: Vec<usize>,
    data: Register,
    dtype: String,
}

#[derive(Serialize)]
pub struct ExecutionResults {
    buffers: HashMap<String, ExecutionResult>,
    execution_duration_microseconds: Option<u64>,
}

/// Fetches results for the corresponding job
pub async fn retrieve_results(
    job_id: &str,
    quantum_processor_id: &str,
    config: &Configuration,
) -> Result<ExecutionResults, runner::Error> {
    let engagement = engagement::get(String::from(quantum_processor_id), config)
        .await
        .map_err(|e| runner::Error::Qpu(format!("Unable to get engagement: {:?}", e)))?;
    let rpcq_client = Client::try_from(&engagement)
        .map_err(|e| {
            warn!("Unable to connect to QPU via RPCQ: {:?}", e);
            runner::Error::Qpu(format!("Unable to connect to QPU via RPCQ: {:?}", e))
        })
        .map(Mutex::new)?;

    let client = rpcq_client.lock().unwrap();
    let results = runner::retrieve_results(JobId(job_id.to_string()), &client).unwrap();
    let execution_duration_microseconds = results.execution_duration_microseconds;
    let buffers = results
        .buffers
        .into_iter()
        .map(|(name, buffer)| {
            let shape = buffer.shape.clone();
            let dtype = buffer.dtype.to_string();
            let data = Register::from(qpu::runner::Register::try_from(buffer).unwrap());
            (name, ExecutionResult { shape, data, dtype })
        })
        .collect::<HashMap<_, _>>();

    Ok(ExecutionResults {
        buffers,
        execution_duration_microseconds,
    })
}
