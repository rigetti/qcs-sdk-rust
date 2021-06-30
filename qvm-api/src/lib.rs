#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![forbid(unsafe_code)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Take in a Quil program as a string `program` and attempt to run it on QVM.
///
/// QVM must be available at <http://localhost:5000/>
///
/// # Errors
/// See [`QVMError`] for possible errors that can occur.
pub async fn run_program_on_qvm(program: &str, shots: u32) -> Result<QVMResponse, QVMError> {
    let request = QVMRequest::new(program, shots);

    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:5000")
        .json(&request)
        .send()
        .await?;

    Ok(response.json().await?)
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
    ConnectionError(#[from] reqwest::Error),
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct QVMRequest {
    quil_instructions: String,
    addresses: HashMap<String, bool>,
    trials: u32,
    #[serde(rename = "type")]
    request_type: RequestType,
}

impl QVMRequest {
    fn new(program: &str, shots: u32) -> Self {
        let mut addresses = HashMap::new();
        addresses.insert("ro".to_string(), true);
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
        let request = QVMRequest::new(program, 1);
        assert_eq!(&request.quil_instructions, program);
    }

    #[test]
    fn it_uses_kebab_case_for_json() {
        let request = QVMRequest::new("H 0", 10);
        let json_string = serde_json::to_string(&request).expect("Could not serialize QVMRequest");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&json_string).unwrap(),
            serde_json::json!({"type": "multishot", "addresses": {"ro": true}, "trials": 10, "quil-instructions": "H 0"})
        )
    }
}
