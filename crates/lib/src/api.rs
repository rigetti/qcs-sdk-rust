use std::{collections::HashMap, str::FromStr};

use qcs_api_client_grpc::{
    models::controller::{readout_values, ControllerJobExecutionResult},
    services::controller::{
        get_controller_job_results_request::Target, GetControllerJobResultsRequest,
    },
};
use quil_rs::expression::Expression;
use serde::Serialize;

use crate::qpu::{
    self,
    client::{ClientGrpcError, QcsClient},
    quilc::{self, TargetDevice},
    rewrite_arithmetic::{self, Substitutions},
    runner,
    translation::{self, EncryptedTranslationResult},
    IsaError,
};

/// Uses quilc to convert a Quil program to native Quil
pub fn compile(
    quil: &str,
    target: TargetDevice,
    client: &QcsClient,
) -> Result<String, Box<dyn std::error::Error>> {
    quilc::compile_program(quil, target, client)
        .map_err(Into::into)
        .map(|p| p.to_string(true))
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

#[derive(Debug, thiserror::Error)]
pub enum TranslationError {
    #[error("Could not translate quil: {0}")]
    Translate(#[from] ClientGrpcError),
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
    client: &QcsClient,
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
    client: &QcsClient,
) -> Result<String, SubmitError> {
    // Is there a better way to map these patch_values keys? This
    // negates the whole purpose of [`submit`] using `Box<str>`,
    // instead of `String` directly, which normally would decrease
    // copies _and_ require less space, since str can't be extended.
    let patch_values = patch_values
        .into_iter()
        .map(|(k, v)| (k.into_boxed_str(), v))
        .collect();

    let job = serde_json::from_str(program).map_err(SubmitError::DeserializeError)?;
    let job_id = runner::submit(quantum_processor_id, job, &patch_values, client).await?;

    Ok(job_id.0)
}

#[derive(Debug, thiserror::Error)]
pub enum SubmitError {
    #[error("Failed to fetch ISA: {0}")]
    IsaError(#[from] IsaError),

    #[error("Failed a gRPC call: {0}")]
    GrpcError(#[from] ClientGrpcError),

    #[error("Failed quilc compilation: {0}")]
    QuilcError(#[from] quilc::Error),

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
                shape: vec![1, c.values.len()],
                dtype: "complex".into(),
                data: Register::Complex64(
                    c.values
                        .iter()
                        .map(|c| [c.real.unwrap_or(0.0), c.imaginary.unwrap_or(0.0)])
                        .collect(),
                ),
            },
            readout_values::Values::IntegerValues(i) => Self {
                shape: vec![1, i.values.len()],
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

/// Fetches results for the corresponding job
pub async fn retrieve_results(
    job_id: &str,
    quantum_processor_id: &str,
    client: &QcsClient,
) -> Result<ExecutionResults, ClientGrpcError> {
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
        .ok_or_else(|| ClientGrpcError::ResponseEmpty("Controller Job Execution Results".into()))
}
