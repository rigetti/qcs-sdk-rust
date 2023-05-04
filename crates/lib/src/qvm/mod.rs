//! This module contains all the functionality for running Quil programs on a QVM. Specifically,
//! the [`Execution`] struct in this module.

use std::{borrow::Cow, collections::HashMap};

use quil_rs::{program::ProgramError, Program};
use serde::{Deserialize, Serialize};

pub(crate) use execution::Execution;

use crate::RegisterData;

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

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub(super) enum MultishotResponse {
    Success(MultishotSuccess),
    Failure(Failure),
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub(super) struct MultishotSuccess {
    #[serde(flatten)]
    pub(super) registers: HashMap<String, RegisterData>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub(super) struct Failure {
    /// The message from QVM describing what went wrong.
    pub(super) status: String,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
struct MultishotRequest<'request> {
    quil_instructions: String,
    addresses: HashMap<&'request str, bool>,
    trials: u16,
    #[serde(rename = "type")]
    request_type: RequestType,
}

impl<'request> MultishotRequest<'request> {
    fn new(program: &str, shots: u16, readouts: &'request [Cow<'request, str>]) -> Self {
        let addresses: HashMap<&str, bool> = readouts.iter().map(|v| (v.as_ref(), true)).collect();
        Self {
            quil_instructions: program.to_string(),
            addresses,
            trials: shots,
            request_type: RequestType::Multishot,
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
struct MultishotMeasureRequest {
    quil_instructions: String,
    trials: u16,
    // Qubits to measure
    qubits: Vec<u64>,
    // Simulated measurement noise for the X, Y, and Z axes.
    measurement_noise: Option<(f64, f64, f64)>,
    // Seed for the random number generator
    rng_seed: Option<i64>,
    #[serde(rename = "type")]
    request_type: RequestType,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub(super) enum MultishotMeasureResponse {
    Success(MultishotSuccess),
    Failure(Failure),
}

impl MultishotMeasureRequest {
    fn new(
        program: &str,
        shots: u16,
        qubits: Vec<u64>,
        measurement_noise: Option<(f64, f64, f64)>,
        rng_seed: Option<i64>,
    ) -> Self {
        Self {
            quil_instructions: program.to_string(),
            trials: shots,
            qubits,
            measurement_noise,
            rng_seed,
            request_type: RequestType::MultishotMeasure,
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
struct ExpectationRequest {
    state_preparation: String,
    operators: Vec<String>,
    rng_seed: Option<i64>,
    #[serde(rename = "type")]
    request_type: RequestType,
}

impl ExpectationRequest {
    fn new(state_preparation: &str, operators: Vec<String>, rng_seed: Option<i64>) -> Self {
        Self {
            state_preparation: state_preparation.to_string(),
            operators,
            rng_seed,
            request_type: RequestType::Expectation,
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
struct WavefunctionRequest {
    compiled_quil: String,
    measurement_noise: Option<(f64, f64, f64)>,
    gate_noise: Option<(f64, f64, f64)>,
    rng_seed: Option<i64>,
    #[serde(rename = "type")]
    request_type: RequestType,
}

impl WavefunctionRequest {
    fn new(
        compiled_quil: &str,
        measurement_noise: Option<(f64, f64, f64)>,
        gate_noise: Option<(f64, f64, f64)>,
        rng_seed: Option<i64>,
    ) -> Self {
        Self {
            compiled_quil: compiled_quil.to_string(),
            measurement_noise,
            gate_noise,
            rng_seed,
            request_type: RequestType::Wavefunction,
        }
    }
}

#[derive(Serialize, Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum RequestType {
    Multishot,
    MultishotMeasure,
    Expectation,
    Wavefunction,
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

#[cfg(test)]
mod describe_request {
    use std::borrow::Cow;

    use super::MultishotRequest;

    #[test]
    fn it_includes_the_program() {
        let program = "H 0";
        let request = MultishotRequest::new(program, 1, &[]);
        assert_eq!(&request.quil_instructions, program);
    }

    #[test]
    fn it_uses_kebab_case_for_json() {
        let request = MultishotRequest::new("H 0", 10, &[Cow::Borrowed("ro")]);
        let json_string = serde_json::to_string(&request).expect("Could not serialize QVMRequest");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&json_string).unwrap(),
            serde_json::json!({"type": "multishot", "addresses": {"ro": true}, "trials": 10, "quil-instructions": "H 0"})
        );
    }
}
