use std::collections::HashMap;
use std::convert::TryFrom;
use std::str::FromStr;

use eyre::{Report, Result, WrapErr};
use quil::Program;
use serde::{Deserialize, Serialize};

use isa::CompilerIsa;
use qcs_api::models::InstructionSetArchitecture;

use crate::configuration::Configuration;

use super::rpcq::{Client as RPCClient, RPCRequest};

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
    isa: &InstructionSetArchitecture,
    config: &Configuration,
) -> Result<NativeQuil> {
    let endpoint = &config.quilc_url;
    let params =
        QuilcParams::new(quil, isa).wrap_err("When creating parameters to send to Quilc")?;
    let request = RPCRequest::new("quil_to_native_quil", &params);
    RPCClient::new(endpoint)
        .wrap_err("When connecting to Quilc")?
        .run_request::<_, QuilcResponse>(&request)
        .map(|response| NativeQuil(response.quil))
        .wrap_err("When sending program to Quilc")
}

/// A wrapper around a [`String`] which indicates the string contains valid Native Quil. That is,
/// Quil which has been processed through `quilc`.
pub struct NativeQuil(String);

impl From<NativeQuil> for String {
    fn from(native_quil: NativeQuil) -> String {
        native_quil.0
    }
}

/// A wrapper around [`Program`] which indicates it has been converted to `NativeQuil` (has been run
/// through `quilc` and therefore is suitable to use on QPUs.
#[derive(Debug, PartialEq)]
pub struct NativeQuilProgram(Program);

impl TryFrom<NativeQuil> for NativeQuilProgram {
    type Error = Report;

    fn try_from(native_quil: NativeQuil) -> Result<Self> {
        let program = Program::from_str(&String::from(native_quil))?;
        Ok(Self(program))
    }
}

impl From<NativeQuilProgram> for Program {
    fn from(program: NativeQuilProgram) -> Program {
        program.0
    }
}

#[derive(Deserialize)]
struct QuilcResponse {
    pub quil: String,
}

/// The top level params that get passed to quilc
#[derive(Serialize, Debug)]
struct QuilcParams {
    protoquil: Option<bool>,
    #[serde(rename = "*args")]
    args: [NativeQuilRequest; 1],
}

impl QuilcParams {
    fn new(quil: &str, isa: &InstructionSetArchitecture) -> Result<Self> {
        Ok(Self {
            protoquil: None,
            args: [NativeQuilRequest::new(quil, isa)?],
        })
    }
}

/// The expected request structure for sending Quil to quilc to be compiled
#[derive(Serialize, Debug)]
#[serde(tag = "_type")]
struct NativeQuilRequest {
    quil: String,
    target_device: TargetDevice,
}

impl NativeQuilRequest {
    fn new(quil: &str, isa: &InstructionSetArchitecture) -> Result<Self> {
        Ok(Self {
            quil: String::from(quil),
            target_device: TargetDevice::try_from(isa)?,
        })
    }
}

/// Description of a device to compile for, part of [`NativeQuilRequest`]
#[derive(Serialize, Debug)]
#[serde(tag = "_type")]
struct TargetDevice {
    isa: CompilerIsa,
    specs: HashMap<String, String>,
}

impl TryFrom<&InstructionSetArchitecture> for TargetDevice {
    type Error = Report;

    fn try_from(isa: &InstructionSetArchitecture) -> Result<Self> {
        Ok(Self {
            isa: CompilerIsa::try_from(isa)
                .wrap_err("When converting ISA to a form that Quilc can understand")?,
            specs: HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use qcs_api::models::InstructionSetArchitecture;

    use super::*;

    const EXPECTED_H0_OUTPUT: &str = r#"MEASURE 0                               # Entering rewiring: #(0 1)
HALT                                    # Exiting rewiring: #(0 1)
"#;

    fn aspen_9_isa() -> InstructionSetArchitecture {
        serde_json::from_reader(File::open("tests/aspen_9_isa.json").unwrap()).unwrap()
    }

    pub fn qvm_isa() -> InstructionSetArchitecture {
        serde_json::from_reader(File::open("tests/qvm_isa.json").unwrap()).unwrap()
    }

    #[test]
    fn compare_native_quil_to_expected_output() {
        let output = compile_program("MEASURE 0", &qvm_isa(), &Configuration::default())
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
        let config = Configuration::default();
        let output =
            compile_program(BELL_STATE, &aspen_9_isa(), &config).expect("Could not compile");
        let results = crate::qvm::Execution::new(&String::from(output))
            .unwrap()
            .run(10, "ro", &Default::default(), &Configuration::default())
            .await
            .expect("Could not run program on QVM");
        for shot in results.into_i8().unwrap() {
            assert_eq!(shot.len(), 2);
            assert_eq!(shot[0], shot[1]);
        }
    }
}
