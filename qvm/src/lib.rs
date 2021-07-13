#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![forbid(unsafe_code)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Take in a Quil program as a string `program` and attempt to run it on QVM.
///
/// QVM must be available at `config.qvm_url`.
///
/// # Arguments
/// 1. `program`: A string of a valid Quil program to run on QVM.
/// 2. `shots`: The number of times the program should run.
/// 3. `register`: The name of the register containing results that should be read out from QVM.
///
/// # Errors
/// See [`QVMError`] for possible errors that can occur.
pub async fn run_program(
    program: &str,
    shots: u16,
    register: &str,
) -> Result<Vec<Vec<u8>>, QVMError> {
    if shots == 0 {
        return Err(QVMError::RegisterMissing);
    }
    let config = qcs_util::get_configuration().await?;
    let request = QVMRequest::new(program, shots, register);

    let client = reqwest::Client::new();
    let response = client.post(&config.qvm_url).json(&request).send().await?;

    let QVMResponse { mut registers } = response.json().await?;
    let mut data = registers
        .remove(register)
        .ok_or(QVMError::RegisterMissing)?;

    if data.len() != shots as usize {
        return Err(QVMError::ShotsMismatch);
    }
    data.shrink_to_fit();
    let shot_len = data[0].len();
    for shot in &mut data {
        if shot.len() != shot_len {
            return Err(QVMError::InconsistentShots);
        }
        shot.shrink_to_fit();
    }

    Ok(data)
}

/// The return value of [`run_program_on_qvm`] if it is successful.
#[derive(Debug, Deserialize)]
pub struct QVMResponse {
    #[serde(flatten)]
    pub registers: HashMap<String, Vec<Vec<u8>>>,
}

/// The return value of [`run_program_on_qvm`] if there is an error.
#[derive(Error, Debug)]
pub enum QVMError {
    #[error("Unable to connect to QVM")]
    Connection(#[from] reqwest::Error),
    #[error("Could not read configuration")]
    Configuration(#[from] qcs_util::ConfigError),
    #[error("The specified register was not present in the results from QVM")]
    RegisterMissing,
    #[error("The returned number of shots from QVM does not match the requested number")]
    ShotsMismatch,
    #[error("Not every shot contained the same amount of data.")]
    InconsistentShots,
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
    use crate::QVMRequest;

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
