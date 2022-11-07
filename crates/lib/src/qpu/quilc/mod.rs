//! This module provides the functions and types necessary to compile a program
//! using quilc.

use std::collections::HashMap;
use std::convert::TryFrom;

use quil_rs::program::{Program, ProgramError};
use serde::{Deserialize, Serialize};

use isa::Compiler;
use qcs_api_client_openapi::models::InstructionSetArchitecture;

use super::{rpcq, Qcs};

mod isa;

/// Number of seconds to wait before timing out.
pub const DEFAULT_COMPILER_TIMEOUT: u8 = 30;

/// Take in a Quil program and produce a "native quil" output from quilc
///
/// # Arguments
///
/// * `program`: The Quil program to compile.
/// * `isa`: The [`InstructionSetArchitecture`] of the targeted platform. Get this using
///     [`super::get_isa`].
/// * `timeout`: The number of seconds to wait before timing out. If not set, defaults to [`DEFAULT_COMPILER_TIMEOUT`].
///
/// returns: `eyre::Result<quil_rs::Program>`
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
    client: &Qcs,
    options: CompilerOpts,
) -> Result<quil_rs::Program, Error> {
    let config = client.get_config();
    let endpoint = config.quilc_url();
    let params = QuilcParams::new(quil, isa);
    let request =
        rpcq::RPCRequest::new("quil_to_native_quil", &params).with_timeout(options.timeout);
    let rpcq_client = rpcq::Client::new(endpoint)
        .map_err(|source| Error::from_quilc_error(endpoint.into(), source))?;
    match rpcq_client.run_request::<_, QuilcCompileProgramResponse>(&request) {
        Ok(response) => response
            .quil
            .parse::<quil_rs::Program>()
            .map_err(Error::Parse),
        Err(source) => Err(Error::from_quilc_error(endpoint.into(), source)),
    }
}

/// A set of options that determine the behavior of compiling programs with quilc
#[derive(Clone, Copy, Debug)]
pub struct CompilerOpts {
    /// The number of seconds to wait before timing out. If `None`, there is no timeout.
    timeout: Option<u8>,
}

/// Functions for building a [`CompilerOpts`] instance
impl CompilerOpts {
    /// Creates a new instance of [`CompilerOpts`] with zero values for each option.
    /// Consider using [`CompilerOpts::default()`] to create an instance with recommended defaults.
    #[must_use]
    pub fn new() -> Self {
        Self { timeout: None }
    }

    /// Set the number of seconds to wait before timing out. If set to None, the timeout is disabled.
    #[must_use]
    pub fn with_timeout(&mut self, seconds: Option<u8>) -> Self {
        self.timeout = seconds;
        *self
    }
}

impl Default for CompilerOpts {
    /// Default compiler options
    /// * `timeout`: See [`DEFAULT_COMPILER_TIMEOUT`]
    fn default() -> Self {
        Self {
            timeout: Some(DEFAULT_COMPILER_TIMEOUT),
        }
    }
}

pub(crate) fn get_version_info(client: &Qcs) -> Result<String, Error> {
    let config = client.get_config();
    let endpoint = config.quilc_url();
    let binding: HashMap<String, String> = HashMap::new();
    let request = rpcq::RPCRequest::new("get_version_info", &binding);
    let rpcq_client = rpcq::Client::new(endpoint)
        .map_err(|source| Error::from_quilc_error(endpoint.into(), source))?;
    match rpcq_client.run_request::<_, QuilcVersionResponse>(&request) {
        Ok(response) => Ok(response.quilc),
        Err(source) => Err(Error::from_quilc_error(endpoint.into(), source)),
    }
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
    /// An error when trying to parse the compiled program.
    #[error("Problem when trying to parse the compiled program: {0}")]
    Parse(ProgramError<Program>),
}

impl Error {
    fn from_quilc_error(quilc_uri: String, source: rpcq::Error) -> Self {
        match source {
            rpcq::Error::Response(message) => Error::QuilcCompilation(message),
            source => Error::QuilcConnection(quilc_uri, source),
        }
    }
}

#[derive(Clone, Deserialize, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct QuilcCompileProgramResponse {
    pub quil: String,
}

#[derive(Clone, Deserialize, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct QuilcVersionResponse {
    pub quilc: String,
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
    use qcs_api_client_openapi::models::InstructionSetArchitecture;
    use regex::Regex;
    use std::fs::File;

    const EXPECTED_H0_OUTPUT: &str = "MEASURE 0\n";

    fn aspen_9_isa() -> InstructionSetArchitecture {
        serde_json::from_reader(File::open("tests/aspen_9_isa.json").unwrap()).unwrap()
    }

    pub fn qvm_isa() -> InstructionSetArchitecture {
        serde_json::from_reader(File::open("tests/qvm_isa.json").unwrap()).unwrap()
    }

    #[tokio::test]
    async fn compare_native_quil_to_expected_output() {
        let output = compile_program(
            "MEASURE 0",
            TargetDevice::try_from(qvm_isa()).expect("Couldn't build target device from ISA"),
            &Qcs::load().await.unwrap_or_default(),
            CompilerOpts::default(),
        )
        .expect("Could not compile");
        assert_eq!(output.to_string(true), EXPECTED_H0_OUTPUT);
    }

    const BELL_STATE: &str = r##"DECLARE ro BIT[2]

H 0
CNOT 0 1

MEASURE 0 ro[0]
MEASURE 1 ro[1]
"##;

    #[tokio::test]
    async fn run_compiled_bell_state_on_qvm() {
        let client = Qcs::load().await.unwrap_or_default();
        let output = compile_program(
            BELL_STATE,
            TargetDevice::try_from(aspen_9_isa()).expect("Couldn't build target device from ISA"),
            &client,
            CompilerOpts::default(),
        )
        .expect("Could not compile");
        let mut results = crate::qvm::Execution::new(&output.to_string(true))
            .unwrap()
            .run(10, &["ro"], &HashMap::default(), &client.get_config())
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

    #[tokio::test]
    async fn get_version_info_from_quilc() {
        let client = Qcs::load().await.unwrap_or_default();
        let version = get_version_info(&client).expect("Should get version info from quilc");
        let semver_re = Regex::new(r"^([0-9]+)\.([0-9]+)\.([0-9]+)$").unwrap();
        assert!(semver_re.is_match(&version));
    }
}
