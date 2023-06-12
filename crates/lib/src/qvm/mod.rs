//! This module contains all the functionality for running Quil programs on a QVM. Specifically,
//! the [`Execution`] struct in this module.

use std::{collections::HashMap, num::NonZeroU16, str::FromStr, time::Duration};

use quil_rs::{
    instruction::{ArithmeticOperand, Instruction, MemoryReference, Move},
    program::ProgramError,
    Program,
};
use serde::Deserialize;

pub(crate) use execution::Execution;

use crate::{client::Qcs, executable::Parameters, RegisterData};

use self::api::AddressRequest;

pub mod api;
mod execution;

/// Number of seconds to wait before timing out.
const DEFAULT_QVM_TIMEOUT: Duration = Duration::from_secs(30);

/// Encapsulates data returned after running a program on the QVM
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Deserialize, Clone, PartialEq)]
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
pub async fn run(
    quil: &str,
    shots: NonZeroU16,
    addresses: HashMap<String, AddressRequest>,
    params: &Parameters,
    measurement_noise: Option<(f64, f64, f64)>,
    gate_noise: Option<(f64, f64, f64)>,
    rng_seed: Option<i64>,
    client: &Qcs,
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
pub async fn run_program(
    program: &Program,
    shots: NonZeroU16,
    addresses: HashMap<String, AddressRequest>,
    params: &Parameters,
    measurement_noise: Option<(f64, f64, f64)>,
    gate_noise: Option<(f64, f64, f64)>,
    rng_seed: Option<i64>,
    client: &Qcs,
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
    let request = api::MultishotRequest::new(
        program.to_string(),
        shots,
        addresses,
        measurement_noise,
        gate_noise,
        rng_seed,
    );
    api::run(&request, client, options)
        .await
        .map(|response| QvmResultData::from_memory_map(response.registers))
}

/// Returns a copy of the [`Program`] with the given parameters applied to it.
/// These parameters are expressed as `MOVE` statements prepended to the program.
pub fn apply_parameters_to_program(
    program: &Program,
    params: &Parameters,
) -> Result<Program, Error> {
    let mut program = program.clone();

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

    program.instructions = params
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
        .chain(program.instructions)
        .collect();

    Ok(program)
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

    use quil_rs::Program;
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

        insta::assert_snapshot!(parameterized_program.to_string());
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
