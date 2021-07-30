//! This module contains all the functionality for running Quil programs on a QVM. Specifically,
//! the [`run_program`] function in this module.

use std::collections::HashMap;

use eyre::{eyre, Result, WrapErr};
use serde::{Deserialize, Serialize};

use crate::configuration::Configuration;
use crate::ProgramResult;

/// Take in a Quil program as a string `program` and attempt to run it on QVM.
///
/// QVM must be available at `config.qvm_url`.
///
/// # Arguments
///
/// 1. `program`: A string of a valid Quil program to run on QVM.
/// 2. `shots`: The number of times the program should run.
/// 3. `register`: The name of the register containing results that should be read out from QVM.
///
/// Returns: [`ProgramResult`].
///
/// # Errors
///
/// All errors are returned in a human-readable format using `eyre` since usually they aren't
/// recoverable at runtime and should just be logged for handling manually.
///
/// ## QVM Connection Errors
///
/// QVM must be running and accessible for this function to succeed. The address can be defined by
/// the `<profile>.applications.pyquil.qvm_url` setting in your QCS `settings.toml`. More info on
/// configuration in [`crate::configuration`].
///
/// ## Execution Errors
///
/// A number of errors could occur if `program` is malformed.
pub async fn run_program(program: &str, shots: u16, register: &str) -> Result<ProgramResult> {
    if shots == 0 {
        return Err(eyre!("A non-zero number of shots must be provided."));
    }
    let config = Configuration::load()
        .await
        .unwrap_or_else(|_| Configuration::default());
    let request = QVMRequest::new(program, shots, register);

    let client = reqwest::Client::new();
    let response = client
        .post(&config.qvm_url)
        .json(&request)
        .send()
        .await
        .wrap_err("While sending data to the QVM")?;

    let QVMResponse { mut registers } = response
        .error_for_status()
        .wrap_err("Received error status from QVM")?
        .json()
        .await
        .wrap_err("While decoding QVM response")?;
    registers.remove(register).ok_or_else(|| {
        eyre!(
            "Could not find register {} in the QVM response, did you measure to it?",
            register
        )
    })
}

#[derive(Debug, Deserialize)]
pub(crate) struct QVMResponse {
    #[serde(flatten)]
    pub registers: HashMap<String, ProgramResult>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct QVMRequest {
    quil_instructions: String,
    addresses: HashMap<String, bool>,
    trials: u16,
    #[serde(rename = "type")]
    request_type: RequestType,
}

impl QVMRequest {
    fn new(program: &str, shots: u16, register: &str) -> Self {
        let mut addresses = HashMap::new();
        addresses.insert(register.to_string(), true);
        Self {
            quil_instructions: program.to_string(),
            addresses,
            trials: shots,
            request_type: RequestType::Multishot,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum RequestType {
    Multishot,
}

#[cfg(test)]
mod describe_qvm_request {
    use super::*;

    #[test]
    fn it_includes_the_program() {
        let program = "H 0";
        let request = QVMRequest::new(program, 1, "ro");
        assert_eq!(&request.quil_instructions, program);
    }

    #[test]
    fn it_uses_kebab_case_for_json() {
        let request = QVMRequest::new("H 0", 10, "ro");
        let json_string = serde_json::to_string(&request).expect("Could not serialize QVMRequest");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&json_string).unwrap(),
            serde_json::json!({"type": "multishot", "addresses": {"ro": true}, "trials": 10, "quil-instructions": "H 0"})
        )
    }
}
