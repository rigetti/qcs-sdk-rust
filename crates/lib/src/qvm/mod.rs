//! This module contains all the functionality for running Quil programs on a QVM. Specifically,
//! the [`Execution`] struct in this module.

use std::{collections::HashMap, num::NonZeroU16, str::FromStr, sync::Arc, time::Duration};

use quil_rs::{
    instruction::{ArithmeticOperand, Instruction, MemoryReference, Move},
    program::ProgramError,
    quil::{Quil, ToQuilError},
    Program,
};
use serde::{Deserialize, Serialize};

pub(crate) use execution::Execution;

use crate::{executable::Parameters, RegisterData};

use self::http::AddressRequest;

mod execution;
pub mod http;
#[cfg(feature = "libquil")]
pub mod libquil;

/// Number of seconds to wait before timing out.
const DEFAULT_QVM_TIMEOUT: Duration = Duration::from_secs(30);

/// Methods supported by the QVM
#[async_trait::async_trait]
pub trait Client {
    /// The QVM version string. Not guaranteed to comply to the semver spec.
    async fn get_version_info(&self, options: &QvmOptions) -> Result<String, Error>;
    /// Execute a program on the QVM.
    async fn run(
        &self,
        request: &http::MultishotRequest,
        options: &QvmOptions,
    ) -> Result<http::MultishotResponse, Error>;
    /// Execute a program on the QVM.
    ///
    /// The behavior of this method is different to that of [`Self::run`]
    /// in that [`Self::run_and_measure`] will execute the program a single
    /// time; the resulting wavefunction is then sampled some number of times
    /// (specified in [`http::MultishotMeasureRequest`]).
    ///
    /// This can be useful if the program is expensive to execute and does
    /// not change per "shot".
    async fn run_and_measure(
        &self,
        request: &http::MultishotMeasureRequest,
        options: &QvmOptions,
    ) -> Result<Vec<Vec<i64>>, Error>;
    /// Measure the expectation value of a program
    async fn measure_expectation(
        &self,
        request: &http::ExpectationRequest,
        options: &QvmOptions,
    ) -> Result<Vec<f64>, Error>;
    /// Get the wavefunction produced by a program
    async fn get_wavefunction(
        &self,
        request: &http::WavefunctionRequest,
        options: &QvmOptions,
    ) -> Result<Vec<u8>, Error>;
}

#[async_trait::async_trait]
impl<T: Client + Sync + Send> Client for Arc<T> {
    async fn get_version_info(&self, options: &QvmOptions) -> Result<String, Error> {
        self.as_ref().get_version_info(options).await
    }

    async fn run(
        &self,
        request: &http::MultishotRequest,
        options: &QvmOptions,
    ) -> Result<http::MultishotResponse, Error> {
        self.as_ref().run(request, options).await
    }

    async fn run_and_measure(
        &self,
        request: &http::MultishotMeasureRequest,
        options: &QvmOptions,
    ) -> Result<Vec<Vec<i64>>, Error> {
        self.as_ref().run_and_measure(request, options).await
    }

    async fn measure_expectation(
        &self,
        request: &http::ExpectationRequest,
        options: &QvmOptions,
    ) -> Result<Vec<f64>, Error> {
        self.as_ref().measure_expectation(request, options).await
    }

    async fn get_wavefunction(
        &self,
        request: &http::WavefunctionRequest,
        options: &QvmOptions,
    ) -> Result<Vec<u8>, Error> {
        self.as_ref().get_wavefunction(request, options).await
    }
}

/// Encapsulates data returned after running a program on the QVM
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct QvmResultData {
    pub(crate) memory: HashMap<String, RegisterData>,
}

impl QvmResultData {
    #[must_use]
    /// Build a [`QvmResultData`] from a mapping of register names to a [`RegisterData`]
    pub fn from_memory_map(memory: HashMap<String, RegisterData>) -> Self {
        Self { memory }
    }

    /// Get a map of register names (ie. "ro") to a [`RegisterData`] containing their values.
    #[must_use]
    pub fn memory(&self) -> &HashMap<String, RegisterData> {
        &self.memory
    }
}

/// Run a Quil program on the QVM. The given parameters are used to parameterize the value of
/// memory locations across shots.
#[allow(clippy::too_many_arguments)]
pub async fn run<C: Client + Send + Sync + ?Sized>(
    quil: &str,
    shots: NonZeroU16,
    addresses: HashMap<String, AddressRequest>,
    params: &Parameters,
    measurement_noise: Option<(f64, f64, f64)>,
    gate_noise: Option<(f64, f64, f64)>,
    rng_seed: Option<i64>,
    client: &C,
    options: &QvmOptions,
) -> Result<QvmResultData, Error> {
    #[cfg(feature = "tracing")]
    tracing::debug!("parsing a program to be executed on the qvm");
    let program = Program::from_str(quil).map_err(Error::Parsing)?;
    run_program(
        &program,
        shots,
        addresses,
        params,
        measurement_noise,
        gate_noise,
        rng_seed,
        client,
        options,
    )
    .await
}

/// Run a [`Program`] on the QVM. The given parameters are used to parametrize the value of
/// memory locations across shots.
#[allow(clippy::too_many_arguments)]
pub async fn run_program<C: Client + ?Sized>(
    program: &Program,
    shots: NonZeroU16,
    addresses: HashMap<String, AddressRequest>,
    params: &Parameters,
    measurement_noise: Option<(f64, f64, f64)>,
    gate_noise: Option<(f64, f64, f64)>,
    rng_seed: Option<i64>,
    client: &C,
    options: &QvmOptions,
) -> Result<QvmResultData, Error> {
    #[cfg(feature = "tracing")]
    tracing::debug!(
        %shots,
        ?addresses,
        ?params,
        "executing program on QVM"
    );
    let program = apply_parameters_to_program(program, params)?;
    let request = http::MultishotRequest::new(
        program.to_quil()?,
        shots,
        addresses,
        measurement_noise,
        gate_noise,
        rng_seed,
    );
    client
        .run(&request, options)
        .await
        .map(|response| QvmResultData::from_memory_map(response.registers))
        .map_err(Into::into)
}

/// Returns a copy of the [`Program`] with the given parameters applied to it.
/// These parameters are expressed as `MOVE` statements prepended to the program.
pub fn apply_parameters_to_program(
    program: &Program,
    params: &Parameters,
) -> Result<Program, Error> {
    let mut new_program = program.clone_without_body_instructions();

    params.iter().try_for_each(|(name, values)| {
        match program.memory_regions.get(name.as_ref()) {
            Some(region) => {
                if region.size.length == values.len() as u64 {
                    Ok(())
                } else {
                    Err(Error::RegionSizeMismatch {
                        name: name.to_string(),
                        declared: region.size.length,
                        parameters: values.len(),
                    })
                }
            }
            None => Err(Error::RegionNotFound { name: name.clone() }),
        }
    })?;

    new_program.add_instructions(
        params
            .iter()
            .flat_map(|(name, values)| {
                values.iter().enumerate().map(move |(index, value)| {
                    Instruction::Move(Move {
                        destination: MemoryReference {
                            name: name.to_string(),
                            index: index as u64,
                        },
                        source: ArithmeticOperand::LiteralReal(*value),
                    })
                })
            })
            .chain(program.body_instructions().cloned())
            .collect::<Vec<_>>(),
    );

    Ok(new_program)
}

/// Options avaialable for running programs on the QVM.
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy, Debug)]
pub struct QvmOptions {
    /// The timeout to use for requests to the QVM. If set to [`None`], there is no timeout.
    pub timeout: Option<Duration>,
}

impl QvmOptions {
    /// Builds a [`QvmOptions`] with the zero value for each option.
    ///
    /// Consider using [`Default`] to get a reasonable set of
    /// configuration options as a starting point.
    #[must_use]
    pub fn new() -> Self {
        Self { timeout: None }
    }
}

impl Default for QvmOptions {
    fn default() -> Self {
        Self {
            timeout: Some(DEFAULT_QVM_TIMEOUT),
        }
    }
}

/// All of the errors that can occur when running a Quil program on QVM.
#[allow(missing_docs)]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error parsing Quil program: {0}")]
    Parsing(#[from] ProgramError),
    #[error("Error converting program to valid Quil: {0}")]
    ToQuil(#[from] ToQuilError),
    #[error("Shots must be a positive integer.")]
    ShotsMustBePositive,
    #[error("Declared region {name} has size {declared} but parameters have size {parameters}.")]
    RegionSizeMismatch {
        name: String,
        declared: u64,
        parameters: usize,
    },
    #[error("Could not find region {name} for parameter. Are you missing a DECLARE instruction?")]
    RegionNotFound { name: Box<str> },
    #[error("Could not communicate with QVM at {qvm_url}")]
    QvmCommunication {
        qvm_url: String,
        source: reqwest::Error,
    },
    #[error("QVM reported a problem running your program: {message}")]
    Qvm { message: String },
    #[error("The client failed to make the request: {0}")]
    Client(#[from] reqwest::Error),
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, str::FromStr};

    use quil_rs::{quil::Quil, Program};
    use rstest::{fixture, rstest};

    use super::apply_parameters_to_program;

    #[fixture]
    fn program() -> Program {
        Program::from_str("DECLARE ro BIT[3]\nH 0").expect("should parse valid program")
    }

    #[rstest]
    fn test_apply_empty_parameters_to_program(program: Program) {
        let parameterized_program = apply_parameters_to_program(&program, &HashMap::new())
            .expect("should not error for empty parameters");

        assert_eq!(parameterized_program, program);
    }

    #[rstest]
    fn test_apply_valid_parameters_to_program(program: Program) {
        let params = HashMap::from([(Box::from("ro"), vec![1.0, 2.0, 3.0])]);
        let parameterized_program = apply_parameters_to_program(&program, &params)
            .expect("should not error for empty parameters");

        insta::assert_snapshot!(parameterized_program.to_quil_or_debug());
    }

    #[rstest]
    fn test_apply_invalid_parameters_to_program(program: Program) {
        let params = HashMap::from([(Box::from("ro"), vec![1.0])]);
        apply_parameters_to_program(&program, &params)
            .expect_err("should error because ro has too few values");

        let params = HashMap::from([(Box::from("ro"), vec![1.0, 2.0, 3.0, 4.0])]);
        apply_parameters_to_program(&program, &params)
            .expect_err("should error because ro has too many values");

        let params = HashMap::from([(Box::from("bar"), vec![1.0])]);
        apply_parameters_to_program(&program, &params)
            .expect_err("should error because bar is not a declared memory region in the program");
    }
}
