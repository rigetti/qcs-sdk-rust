use log::warn;
use qcs_api::models::InstructionSetArchitecture;
use quil_rs::expression::Expression;
use serde::Serialize;

use crate::{
    configuration::Configuration,
    qpu::{
        self, engagement, quilc,
        rewrite_arithmetic::{self, Substitutions},
        rpcq::Client,
        runner::{self, JobId},
        translation,
    },
};
use std::{collections::HashMap, convert::TryFrom, str::FromStr, sync::Mutex};

// TODO Define qcs.get_compiler_isa_from_qcs_isa(qcs_isa: String)
// Quilc expects a "compiler ISA" which is a restructured subset
// of the data in the QCS ISA. Having a SDK utility to convert
// between the two seems less error-prone / surprising to the user
// than trying to accept either type of ISA, or provide kwargs, etc.

/// Uses quilc to convert a Quil program to native Quil
pub async fn compile(
    quil: &str,
    quantum_processor_isa: InstructionSetArchitecture,
    config: &Configuration,
) -> Result<String, Box<dyn std::error::Error>> {
    quilc::compile_program(quil, quantum_processor_isa, config)
        .map_err(|e| e.into())
        .map(|p| p.0)
}

#[derive(Serialize)]
pub struct RewriteArithmeticResult {
    pub program: String,
    pub recalculation_table: Vec<String>,
}

// TODO: real errors here; no unwrap
pub fn rewrite_arithmetic(native_quil: &str) -> Result<RewriteArithmeticResult, String> {
    let program = quil_rs::program::Program::from_str(native_quil).unwrap();

    let (program, subs) = qpu::rewrite_arithmetic::rewrite_arithmetic(program).unwrap();
    let recalculation_table = subs.into_iter().map(|expr| expr.to_string()).collect();

    Ok(RewriteArithmeticResult {
        program: program.to_string(true),
        recalculation_table,
    })
}

#[derive(Clone, Debug, PartialEq, Default, Serialize)]
pub struct TranslationResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_descriptors:
        Option<::std::collections::HashMap<String, qcs_api::models::ParameterSpec>>,
    pub program: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ro_sources: Option<Vec<Vec<String>>>,
    /// ISO8601 timestamp of the settings used to translate the program. Translation is deterministic; a program translated twice with the same settings by the same version of the service will have identical output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings_timestamp: Option<String>,
}

/// Translates a native Quil program into an executable
pub async fn translate(
    native_quil: &str,
    shots: u16,
    quantum_processor_id: &str,
    config: &Configuration,
) -> Result<TranslationResult, translation::Error> {
    let translation = translation::translate(
        qpu::RewrittenQuil(native_quil.to_string()),
        shots,
        quantum_processor_id,
        config,
    )
    .await?;

    Ok(TranslationResult {
        memory_descriptors: translation.memory_descriptors,
        program: translation.program,
        ro_sources: translation.ro_sources,
        settings_timestamp: translation.settings_timestamp,
    })
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

pub fn build_patch_values(
    recalculation_table: Vec<String>,
    memory: HashMap<Box<str>, Vec<f64>>,
) -> Result<HashMap<Box<str>, Vec<f64>>, String> {
    let substitutions: Substitutions = recalculation_table
        .iter()
        .map(|expr| Expression::from_str(expr))
        .collect::<Result<_, _>>()
        .map_err(|e| format!("Unable to interpret recalc table: {:?}", e))?;
    rewrite_arithmetic::get_substitutions(&substitutions, &memory)
}

pub type Complex64 = [f32; 2];

/// Data from an individual register. Each variant contains a vector with the expected data type
/// where each value in the vector corresponds to a shot.
#[derive(Debug, PartialEq, Serialize)]
#[serde(untagged)] // Don't include the discriminant name in serialized output.
pub enum Register {
    F64(Vec<f64>),
    I16(Vec<i16>),
    Complex64(Vec<Complex64>),
    I8(Vec<i8>),
}

impl From<qpu::runner::Register> for Register {
    fn from(register: qpu::runner::Register) -> Self {
        match register {
            runner::Register::F64(f) => Register::F64(f),
            runner::Register::I16(i) => Register::I16(i),
            runner::Register::Complex32(c) => {
                Register::Complex64(c.iter().map(|c| [c.re, c.im]).collect())
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
