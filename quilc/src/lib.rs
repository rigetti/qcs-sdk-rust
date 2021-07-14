#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::convert::TryFrom;

use serde::{Deserialize, Serialize};

use qcs_api::models::InstructionSetArchitecture;

mod isa;

use isa::CompilerIsa;
use rpcq::RPCRequest;

/// Take in a Quil program and produce a "native quil" output from quilc
///
/// # Arguments
///
/// * `program`: The Quil program to compile.
/// * `isa`: The [`InstructionSetArchitecture`] of the targeted platform
///
/// returns: Result<String, [`CompileError`]>
///
/// # Errors
///
/// See [`CompileError`] for details on specific errors.
pub fn compile_program(
    quil: &str,
    isa: &InstructionSetArchitecture,
    config: &qcs_util::Configuration,
) -> Result<NativeQuil, CompileError> {
    let endpoint = &config.quilc_url;
    let params = QuilcParams::new(quil, isa)?;
    let request = RPCRequest::new("quil_to_native_quil", params);
    rpcq::Client::new(endpoint)?
        .run_request::<_, QuilcResponse>(&request)
        .map(|response| NativeQuil(response.quil))
        .map_err(CompileError::from)
}

pub struct NativeQuil(String);

impl From<NativeQuil> for String {
    fn from(native_quil: NativeQuil) -> String {
        native_quil.0
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
    fn new(quil: &str, isa: &InstructionSetArchitecture) -> Result<Self, CompileError> {
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
    fn new(quil: &str, isa: &InstructionSetArchitecture) -> Result<Self, CompileError> {
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
    type Error = CompileError;

    fn try_from(isa: &InstructionSetArchitecture) -> Result<Self, Self::Error> {
        Ok(Self {
            isa: CompilerIsa::try_from(isa)?,
            specs: HashMap::new(),
        })
    }
}

/// The possible errors that can occur when calling [`compile_program`]
#[derive(thiserror::Error, Debug)]
pub enum CompileError {
    /// There was a problem communicating with the quilc API, is it running?
    #[error("Could not communicate with quilc")]
    Communication(#[from] rpcq::Error),
    /// A problem converting the [`InstructionSetArchitecture`] to a quilc-compatible form.
    #[error("Unable to convert the ISA from QCS to something quilc can understand")]
    IsaConversion(#[from] isa::Error),
}
