//! This module provides the functions and types necessary to compile a program
//! using quilc.

use std::collections::HashMap;
use std::convert::TryFrom;
use std::str::FromStr;

use quil_rs::Program;
use serde::{Deserialize, Serialize};

use isa::Compiler;
use qcs_api::models::InstructionSetArchitecture;

use crate::configuration::Configuration;

use super::rpcq;

mod isa;

/// Take in a Quil program and produce a "native quil" output from quilc
///
/// # Arguments
///
/// * `program`: The Quil program to compile.
/// * `isa`: The [`InstructionSetArchitecture`] of the targeted platform. Get this using
///     [`super::get_isa`].
///
/// returns: `eyre::Result<NativeQuil>`
///
/// # Errors
///
/// `eyre` is used to create human-readable error messages, since most of the errors are not
/// recoverable at runtime. This function can fail generally if the provided ISA cannot be converted
/// into a form that `quilc` recognizes, if `quilc` cannot be contacted, or if the program cannot
/// be converted by `quilc`.
pub(crate) fn compile_program(
    quil: &str,
    isa: TargetDevice,
    config: &Configuration,
) -> Result<NativeQuil, Error> {
    let endpoint = &config.quilc_url;
    let params = QuilcParams::new(quil, isa);
    let request = rpcq::RPCRequest::new("quil_to_native_quil", &params);
    rpcq::Client::new(endpoint)
        .map_err(|source| Error::from_quilc_error(endpoint.clone(), source))?
        .run_request::<_, QuilcResponse>(&request)
        .map(|response| NativeQuil(response.quil))
        .map_err(|source| Error::from_quilc_error(endpoint.clone(), source))
}

/// All of the errors that can occur within this module.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An ISA-related error.
    #[error("Problem converting ISA to quilc format. This is a bug in this library or in QCS.")]
    Isa(#[from] isa::Error),
    /// An error when trying to connect to quilc.
    #[error("Problem connecting to quilc at {0}")]
    QuilcConnection(String, #[source] rpcq::Error),
    /// An error when trying to compile using quilc.
    #[error("Problem compiling quil program: {0}")]
    QuilcCompilation(String),
}

impl Error {
    fn from_quilc_error(quilc_uri: String, source: rpcq::Error) -> Self {
        match source {
            rpcq::Error::Response(message) => Error::QuilcCompilation(message),
            source => Error::QuilcConnection(quilc_uri, source),
        }
    }
}

/// A wrapper around a [`String`] which indicates the string contains valid Native Quil. That is,
/// Quil which has been processed through `quilc`.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NativeQuil(String);

impl NativeQuil {
    /// Cast a String to `NativeQuil` without checking or transforming it via `quilc`.
    pub(super) fn assume_native_quil(quil: String) -> Self {
        NativeQuil(quil)
    }
}

impl From<NativeQuil> for String {
    fn from(native_quil: NativeQuil) -> String {
        native_quil.0
    }
}

impl AsRef<str> for NativeQuil {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// A wrapper around [`Program`] which indicates it has been converted to `NativeQuil` (has been run
/// through `quilc` and therefore is suitable to use on QPUs.
#[derive(Debug, PartialEq, Clone)]
pub(super) struct NativeQuilProgram(Program);

impl TryFrom<NativeQuil> for NativeQuilProgram {
    type Error = <Program as FromStr>::Err;

    fn try_from(native_quil: NativeQuil) -> Result<Self, Self::Error> {
        let program = Program::from_str(&String::from(native_quil))?;
        Ok(Self(program))
    }
}

impl From<NativeQuilProgram> for Program {
    fn from(program: NativeQuilProgram) -> Program {
        program.0
    }
}

#[derive(Clone, Deserialize, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct QuilcResponse {
    pub quil: String,
}

/// The top level params that get passed to quilc
#[derive(Serialize, Debug, Clone, PartialEq)]
struct QuilcParams {
    protoquil: Option<bool>,
    #[serde(rename = "*args")]
    args: [NativeQuilRequest; 1],
}

impl QuilcParams {
    fn new(quil: &str, isa: TargetDevice) -> Self {
        Self {
            protoquil: None,
            args: [NativeQuilRequest::new(quil, isa)],
        }
    }
}

/// The expected request structure for sending Quil to quilc to be compiled
#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(tag = "_type")]
struct NativeQuilRequest {
    quil: String,
    target_device: TargetDevice,
}

impl NativeQuilRequest {
    fn new(quil: &str, target_device: TargetDevice) -> Self {
        Self {
            quil: String::from(quil),
            target_device,
        }
    }
}

/// Description of a device to compile for, part of [`NativeQuilRequest`]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "_type")]
pub struct TargetDevice {
    isa: Compiler,
    specs: HashMap<String, String>,
}

impl TryFrom<InstructionSetArchitecture> for TargetDevice {
    type Error = Error;

    fn try_from(isa: InstructionSetArchitecture) -> Result<Self, Self::Error> {
        Ok(Self {
            isa: Compiler::try_from(isa)?,
            specs: HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qcs_api::models::InstructionSetArchitecture;
    use std::fs::File;

    const EXPECTED_H0_OUTPUT: &str =
        "MEASURE 0                               # Entering/exiting rewiring: (#(0 1) . #(0 1))\n";

    fn aspen_9_isa() -> InstructionSetArchitecture {
        serde_json::from_reader(File::open("tests/aspen_9_isa.json").unwrap()).unwrap()
    }

    pub fn qvm_isa() -> InstructionSetArchitecture {
        serde_json::from_reader(File::open("tests/qvm_isa.json").unwrap()).unwrap()
    }

    #[test]
    fn compare_native_quil_to_expected_output() {
        let output = compile_program(
            "MEASURE 0",
            TargetDevice::try_from(qvm_isa()).expect("Couldn't build target device from ISA"),
            &Configuration::default(),
        )
        .expect("Could not compile");
        assert_eq!(String::from(output), EXPECTED_H0_OUTPUT);
    }

    const BELL_STATE: &str = r##"DECLARE ro BIT[2]

H 0
CNOT 0 1

MEASURE 0 ro[0]
MEASURE 1 ro[1]
"##;

    #[tokio::test]
    async fn run_compiled_bell_state_on_qvm() {
        let config = Configuration::load().await.unwrap_or_default();
        let output = compile_program(
            BELL_STATE,
            TargetDevice::try_from(aspen_9_isa()).expect("Couldn't build target device from ISA"),
            &config,
        )
        .expect("Could not compile");
        let mut results = crate::qvm::Execution::new(&String::from(output))
            .unwrap()
            .run(10, &["ro"], &HashMap::default(), &config)
            .await
            .expect("Could not run program on QVM");
        for shot in results
            .remove("ro")
            .expect("Did not receive ro buffer")
            .into_i8()
            .unwrap()
        {
            assert_eq!(shot.len(), 2);
            assert_eq!(shot[0], shot[1]);
        }
    }
}
