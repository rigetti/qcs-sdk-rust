//! This module contains all the functionality for running Quil programs on a QVM. Specifically,
//! the [`Execution`] struct in this module.

use std::{collections::HashMap, str::FromStr};

use qcs_api_client_common::ClientConfiguration;
use quil_rs::{
    instruction::{ArithmeticOperand, Instruction, MemoryReference, Move},
    program::ProgramError,
    Program,
};
use serde::Deserialize;

pub(crate) use execution::Execution;

use crate::{executable::Parameters, RegisterData};

use self::api::AddressRequest;

pub mod api;
mod execution;

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

/// Run a Quil program on the QVM. The given [`Parameters`] are used to parameterize the value of
/// memory locations across shots.
pub async fn run(
    quil: &str,
    shots: u16,
    addresses: HashMap<String, AddressRequest>,
    params: &Parameters,
    config: &ClientConfiguration,
) -> Result<QvmResultData, Error> {
    #[cfg(feature = "tracing")]
    tracing::debug!("parsing a program to be executed on the qvm");
    let program = Program::from_str(quil).map_err(Error::Parsing)?;
    run_program(&program, shots, addresses, params, config).await
}

/// Run a [`Program`] on the QVM. The given [`Parameters`] are used to parametrize the value of
/// memory locations across shots.
pub async fn run_program(
    program: &Program,
    shots: u16,
    addresses: HashMap<String, AddressRequest>,
    params: &Parameters,
    config: &ClientConfiguration,
) -> Result<QvmResultData, Error> {
    #[cfg(feature = "tracing")]
    tracing::debug!(
        %shots,
        ?addresses,
        ?params,
        "executing program on QVM"
    );
    if shots == 0 {
        return Err(Error::ShotsMustBePositive);
    }

    // Create a clone of the program so MOVE statements can be prepended to it
    let mut program = program.clone();

    for (name, values) in params {
        match program.memory_regions.get(name.as_ref()) {
            Some(region) => {
                if region.size.length != values.len() as u64 {
                    return Err(Error::RegionSizeMismatch {
                        name: name.clone(),
                        declared: region.size.length,
                        parameters: values.len(),
                    });
                }
            }
            None => {
                return Err(Error::RegionNotFound { name: name.clone() });
            }
        }
        for (index, value) in values.iter().enumerate() {
            program.instructions.insert(
                0,
                Instruction::Move(Move {
                    destination: ArithmeticOperand::MemoryReference(MemoryReference {
                        name: name.to_string(),
                        index: index as u64,
                    }),
                    source: ArithmeticOperand::LiteralReal(*value),
                }),
            );
        }
    }

    let request = api::MultishotRequest::new(&program.to_string(true), shots, addresses);
    api::run(&request, config)
        .await
        .map(|response| QvmResultData::from_memory_map(response.registers))
}

/// All of the errors that can occur when running a Quil program on QVM.
#[allow(missing_docs)]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error parsing Quil program: {0}")]
    Parsing(#[from] ProgramError<Program>),
    #[error("Shots must be a positive integer.")]
    ShotsMustBePositive,
    #[error("Declared region {name} has size {declared} but parameters have size {parameters}.")]
    RegionSizeMismatch {
        name: Box<str>,
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
}
