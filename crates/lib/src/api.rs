//! This module provides a small interface for doing compilation, translation,
//! and execution.

use log::warn;
use quil_rs::Program;
use serde::Serialize;

use crate::{
    configuration::Configuration,
    qpu::{
        self, engagement,
        quilc::{self, TargetDevice},
        rewrite_arithmetic::{self, Substitutions},
        rpcq::Client,
        runner::{self, JobId},
        translation,
    },
};
use std::{collections::HashMap, convert::TryFrom};

/// Uses quilc to convert a Quil program to native Quil
///
/// # Errors
///
/// See [`quilc::compile_program`].
pub fn compile(
    quil: &str,
    target_device: TargetDevice,
    config: &Configuration,
) -> Result<String, Box<dyn std::error::Error>> {
    quilc::compile_program(quil, target_device, config)
        .map_err(std::convert::Into::into)
        .map(String::from)
}

/// The result of a call to [`rewrite_arithmetic`] which provides the
/// information necessary to later patch-in memory values to a compiled program.
#[derive(Serialize)]
pub struct RewriteArithmeticResult {
    /// The rewritten program
    pub program: String,
    /// The expressions used to fill-in the `__SUBST` memory location. The
    /// expression index in this vec is the same as that in `__SUBST`.
    pub recalculation_table: Vec<String>,
}

/// Rewrite parametric arithmetic such that all gate parameters are only memory
/// references to newly declared memory location (`__SUBST`).
///
/// A "recalculation" table is provided which can be used to populate the memory
/// when needed (see `build_patch_values`).
///
/// # Errors
///
/// May return an error if the program fails to parse, or the parameter arithmetic
/// cannot be rewritten.
pub fn rewrite_arithmetic(native_quil: &str) -> Result<RewriteArithmeticResult, String> {
    let program: Program = native_quil.parse()?;

    let (program, subs) =
        qpu::rewrite_arithmetic::rewrite_arithmetic(program).map_err(|e| e.to_string())?;
    let recalculation_table = subs.into_iter().map(|expr| expr.to_string()).collect();

    Ok(RewriteArithmeticResult {
        program: program.to_string(true),
        recalculation_table,
    })
}

/// The result of a call to [`translate`] which provides information about the
/// translated program.
#[derive(Clone, Debug, PartialEq, Default, Serialize)]
pub struct TranslationResult {
    /// The memory defined in the program.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_descriptors:
        Option<::std::collections::HashMap<String, qcs_api::models::ParameterSpec>>,
    /// The translated program.
    pub program: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The memory locations used for readout.
    pub ro_sources: Option<Vec<Vec<String>>>,
    /// ISO8601 timestamp of the settings used to translate the program. Translation is deterministic; a program translated twice with the same settings by the same version of the service will have identical output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings_timestamp: Option<String>,
}

/// Translates a native Quil program into an executable
///
/// # Errors
///
/// Returns a [`translation::Error`] if translation fails.
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
///
/// # Errors
///
/// May return an error if
/// * an engagement is not available
/// * an RPCQ client cannot be built
/// * the program cannot be submitted
#[allow(clippy::implicit_hasher)]
pub async fn submit(
    program: &str,
    patch_values: HashMap<String, Vec<f64>>,
    quantum_processor_id: &str,
    config: &Configuration,
) -> Result<String, runner::Error> {
    // Is there a better way to map these patch_values keys? This
    // negates the whole purpose of [`submit`] using `Box<str>`,
    // instead of `String` directly, which normally would decrease
    // copies _and_ require less space, since str can't be extended.
    let patch_values = patch_values
        .into_iter()
        .map(|(k, v)| (k.into_boxed_str(), v))
        .collect();
    let engagement = engagement::get(String::from(quantum_processor_id), config)
        .await
        .map_err(|e| runner::Error::Qpu(format!("Unable to get engagement: {:?}", e)))?;
    let rpcq_client = Client::try_from(&engagement).map_err(|e| {
        warn!("Unable to connect to QPU via RPCQ: {:?}", e);
        runner::Error::Connection(e)
    })?;

    let job_id = runner::submit(program, &patch_values, &rpcq_client)?;

    Ok(job_id.0)
}

/// Evaluate the expressions in `recalculation_table` using the numeric values
/// provided in `memory`.
///
/// # Errors
#[allow(clippy::implicit_hasher)]
pub fn build_patch_values(
    recalculation_table: &[String],
    memory: &HashMap<Box<str>, Vec<f64>>,
) -> Result<HashMap<Box<str>, Vec<f64>>, String> {
    let substitutions: Substitutions = recalculation_table
        .iter()
        .map(|expr| expr.parse())
        .collect::<Result<_, _>>()
        .map_err(|e| format!("Unable to interpret recalc table: {:?}", e))?;
    rewrite_arithmetic::get_substitutions(&substitutions, memory)
}

/// A 64-bit complex number.
pub type Complex64 = [f32; 2];

/// Data from an individual register. Each variant contains a vector with the expected data type
/// where each value in the vector corresponds to a shot.
#[derive(Debug, PartialEq, Serialize)]
#[serde(untagged)] // Don't include the discriminant name in serialized output.
pub enum Register {
    /// A register of 64-bit floating point numbers
    F64(Vec<f64>),
    /// A register of 16-bit integers
    I16(Vec<i16>),
    /// A register of 64-bit complex numbers
    Complex64(Vec<Complex64>),
    /// A register of 8-bit integers (bytes)
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

/// The execution readout data from a particular memory location.
#[derive(Serialize)]
pub struct ExecutionResult {
    shape: Vec<usize>,
    data: Register,
    dtype: String,
}

/// Execution readout data for all memory locations.
#[derive(Serialize)]
pub struct ExecutionResults {
    buffers: HashMap<String, ExecutionResult>,
    execution_duration_microseconds: Option<u64>,
}

/// Fetches results for the corresponding job
///
/// # Errors
///
/// May error if an engagement is not available, an RPCQ client cannot be built,
/// or retrieval of results fails.
///
/// # Panics
///
/// Panics if a [`Register`] cannot be constructed from a result buffer.
pub async fn retrieve_results(
    job_id: &str,
    quantum_processor_id: &str,
    config: &Configuration,
) -> Result<ExecutionResults, runner::Error> {
    let engagement = engagement::get(String::from(quantum_processor_id), config)
        .await
        .map_err(|e| runner::Error::Qpu(format!("Unable to get engagement: {:?}", e)))?;
    let rpcq_client = Client::try_from(&engagement).map_err(|e| {
        warn!("Unable to connect to QPU via RPCQ: {:?}", e);
        runner::Error::Qpu(format!("Unable to connect to QPU via RPCQ: {:?}", e))
    })?;

    let results = runner::retrieve_results(JobId(job_id.to_string()), &rpcq_client)?;
    let execution_duration_microseconds = results.execution_duration_microseconds;
    let buffers = results
        .buffers
        .into_iter()
        .map(|(name, buffer)| {
            let shape = buffer.shape.clone();
            let dtype = buffer.dtype.to_string();
            // TODO Get rid of this unwrap.
            let data = Register::from(qpu::runner::Register::try_from(buffer).unwrap());
            (name, ExecutionResult { shape, data, dtype })
        })
        .collect::<HashMap<_, _>>();

    Ok(ExecutionResults {
        buffers,
        execution_duration_microseconds,
    })
}
