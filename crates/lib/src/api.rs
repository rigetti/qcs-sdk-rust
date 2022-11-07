//! This module provides convenience functions to handle compilation,
//! translation, parameter arithmetic rewriting, and results collection.

use std::{collections::HashMap, str::FromStr};

use qcs_api_client_grpc::{
    models::controller::{readout_values, ControllerJobExecutionResult},
    services::controller::{
        get_controller_job_results_request::Target, GetControllerJobResultsRequest,
    },
};
use quil_rs::expression::Expression;
use quil_rs::{program::ProgramError, Program};
use serde::Serialize;

use crate::qpu::{
    self,
    client::{GrpcClientError, Qcs},
    quilc::{self, CompilerOpts, TargetDevice},
    rewrite_arithmetic::{self, Substitutions},
    runner,
    translation::{self, EncryptedTranslationResult},
    IsaError,
};

/// Uses quilc to convert a Quil program to native Quil
/// TODO: Add `+ Send + Sync` to the error type once quil-rs supports it:
/// <https://github.com/rigetti/quil-rs/issues/122>
/// <https://github.com/rigetti/qcs-sdk-rust/issues/210>
pub fn compile(
    quil: &str,
    target: TargetDevice,
    client: &Qcs,
    options: CompilerOpts,
) -> Result<String, Box<dyn std::error::Error + 'static>> {
    quilc::compile_program(quil, target, client, options)
        .map_err(Into::into)
        .map(|p| p.to_string(true))
}

/// Gets the quilc version
/// TODO: Add `+ Send + Sync` to the error type once quil-rs supports it:
/// <https://github.com/rigetti/quil-rs/issues/122>
/// <https://github.com/rigetti/qcs-sdk-rust/issues/210>
pub fn get_quilc_version(client: &Qcs) -> Result<String, Box<dyn std::error::Error + 'static>> {
    quilc::get_version_info(client).map_err(Into::into)
}

/// Collection of errors that can result from rewriting arithmetic.
#[derive(thiserror::Error, Debug)]
pub enum RewriteArithmeticError {
    /// The Quil program could not be parsed.
    #[error("Could not parse program: {0}")]
    Program(#[from] ProgramError<Program>),
    /// Parameteric arithmetic in the Quil program could not be rewritten.
    #[error("Could not rewrite arithmetic: {0}")]
    Rewrite(#[from] rewrite_arithmetic::Error),
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
pub fn rewrite_arithmetic(
    native_quil: quil_rs::Program,
) -> Result<RewriteArithmeticResult, rewrite_arithmetic::Error> {
    let (program, subs) = qpu::rewrite_arithmetic::rewrite_arithmetic(native_quil)?;
    let recalculation_table = subs.into_iter().map(|expr| expr.to_string()).collect();

    Ok(RewriteArithmeticResult {
        program: program.to_string(true),
        recalculation_table,
    })
}

/// Errors that can happen during translation
#[derive(Debug, thiserror::Error)]
pub enum TranslationError {
    /// The program could not be translated
    #[error("Could not translate quil: {0}")]
    Translate(#[from] GrpcClientError),
    /// The result of translation could not be deserialized
    #[error("Could not serialize translation result: {0}")]
    Serialize(#[from] serde_json::Error),
}

/// The result of a call to [`translate`] which provides information about the
/// translated program.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize)]
pub struct TranslationResult {
    /// The translated program.
    pub program: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The memory locations used for readout.
    pub ro_sources: Option<HashMap<String, String>>,
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
    client: &Qcs,
) -> Result<TranslationResult, TranslationError> {
    let EncryptedTranslationResult { job, readout_map } =
        translation::translate(quantum_processor_id, native_quil, shots.into(), client).await?;

    let program = serde_json::to_string(&job)?;

    Ok(TranslationResult {
        ro_sources: Some(readout_map),
        program,
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
    client: &Qcs,
) -> Result<String, SubmitError> {
    // Is there a better way to map these patch_values keys? This
    // negates the whole purpose of [`submit`] using `Box<str>`,
    // instead of `String` directly, which normally would decrease
    // copies _and_ require less space, since str can't be extended.
    let patch_values = patch_values
        .into_iter()
        .map(|(k, v)| (k.into_boxed_str(), v))
        .collect();

    let job = serde_json::from_str(program)?;
    let job_id = runner::submit(quantum_processor_id, job, &patch_values, client).await?;

    Ok(job_id.0)
}

/// Errors that may occur when submitting a program for execution
#[derive(Debug, thiserror::Error)]
pub enum SubmitError {
    /// Failed to fetch the desired ISA
    #[error("Failed to fetch ISA: {0}")]
    IsaError(#[from] IsaError),

    /// Failed a gRPC API call
    #[error("Failed a gRPC call: {0}")]
    GrpcError(#[from] GrpcClientError),

    /// Quilc compilation failed
    #[error("Failed quilc compilation: {0}")]
    QuilcError(#[from] quilc::Error),

    /// Job could not be deserialized
    #[error("Failed to deserialize job: {0}")]
    DeserializeError(#[from] serde_json::Error),
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
        .map(|expr| Expression::from_str(expr))
        .collect::<Result<_, _>>()
        .map_err(|e| format!("Unable to interpret recalculation table: {:?}", e))?;
    rewrite_arithmetic::get_substitutions(&substitutions, memory)
}

/// A convenience type that describes a Complex-64 value whose real
/// and imaginary parts of both f32.
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
    /// A register of 32-bit integers
    I32(Vec<i32>),
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

impl From<readout_values::Values> for ExecutionResult {
    fn from(values: readout_values::Values) -> Self {
        match values {
            readout_values::Values::ComplexValues(c) => Self {
                shape: vec![c.values.len(), 1],
                dtype: "complex".into(),
                data: Register::Complex64(
                    c.values
                        .iter()
                        .map(|c| [c.real.unwrap_or(0.0), c.imaginary.unwrap_or(0.0)])
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
#[derive(Serialize)]
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
/// May error if a [`gRPC`] client cannot be constructed, or a [`gRPC`]
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
