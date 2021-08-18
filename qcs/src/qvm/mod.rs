//! This module contains all the functionality for running Quil programs on a QVM. Specifically,
//! the [`Execution`] struct in this module.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub(crate) use execution::Execution;

use crate::ExecutionResult;

mod execution;

#[derive(Debug, Deserialize)]
struct QVMResponse {
    #[serde(flatten)]
    pub registers: HashMap<String, ExecutionResult>,
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
