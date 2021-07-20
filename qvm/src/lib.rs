#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![forbid(unsafe_code)]

use std::collections::HashMap;

use eyre::{eyre, Result, WrapErr};
use qcs_util::Configuration;
use serde::{Deserialize, Serialize};

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
pub async fn run_program(program: &str, shots: u16, register: &str) -> Result<Vec<Vec<u8>>> {
    if shots == 0 {
        return Err(eyre!("A non-zero number of shots must be provided."));
    }
    let config = qcs_util::get_configuration()
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
        .json()
        .await
        .wrap_err("While decoding QVM response")?;
    let mut data = registers.remove(register).ok_or_else(|| {
        eyre!(
            "Could not find register {} in the QVM response, did you measure to it?",
            register
        )
    })?;

    if data.len() != shots as usize {
        return Err(eyre!(
            "Expected {} shots but received {}",
            shots,
            data.len()
        ));
    }
    data.shrink_to_fit();
    let shot_len = data[0].len();
    for (shot_num, shot) in data.iter_mut().enumerate() {
        if shot.len() != shot_len {
            return Err(eyre!(
                "Each shot must have the same amount of data. However, shot 0 had \
                {shot_len} entries and shot {shot_num} had {this_shot_len} entries.",
                shot_len = shot_len,
                shot_num = shot_num,
                this_shot_len = shot.len(),
            ));
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
