#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::convert::TryFrom;

use eyre::{Report, Result, WrapErr};
use serde::{Deserialize, Serialize};

use isa::CompilerIsa;
use qcs_api::models::InstructionSetArchitecture;
use rpcq::RPCRequest;

mod isa;

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
) -> Result<NativeQuil> {
    let endpoint = &config.quilc_url;
    let params =
        QuilcParams::new(quil, isa).wrap_err("When creating parameters to send to Quilc")?;
    let request = RPCRequest::new("quil_to_native_quil", params);
    rpcq::Client::new(endpoint)
        .wrap_err("When connecting to Quilc")?
        .run_request::<_, QuilcResponse>(&request)
        .map(|response| NativeQuil(response.quil))
        .wrap_err("When sending program to Quilc")
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
