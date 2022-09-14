//! This module contains all the functionality for running Quil programs on a QVM. Specifically,
//! the [`Execution`] struct in this module.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub(crate) use execution::{Error, Execution};

use crate::RegisterData;

mod execution;

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub(super) enum Response {
    Success(Success),
    Failure(Failure),
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub(super) struct Success {
    #[serde(flatten)]
    pub registers: HashMap<String, RegisterData>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub(super) struct Failure {
    /// The message from QVM describing what went wrong.
    pub status: String,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
struct Request<'request> {
    quil_instructions: String,
    addresses: HashMap<&'request str, bool>,
    trials: u16,
    #[serde(rename = "type")]
    request_type: RequestType,
}

impl<'request> Request<'request> {
    fn new(program: &str, shots: u16, readouts: &[&'request str]) -> Self {
        let addresses: HashMap<&str, bool> = readouts.iter().map(|v| (*v, true)).collect();
        Self {
            quil_instructions: program.to_string(),
            addresses,
            trials: shots,
            request_type: RequestType::Multishot,
        }
    }
}

#[derive(Serialize, Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
enum RequestType {
    Multishot,
}

#[cfg(test)]
mod describe_request {
    use super::Request;

    #[test]
    fn it_includes_the_program() {
        let program = "H 0";
        let request = Request::new(program, 1, &[]);
        assert_eq!(&request.quil_instructions, program);
    }

    #[test]
    fn it_uses_kebab_case_for_json() {
        let request = Request::new("H 0", 10, &["ro"]);
        let json_string = serde_json::to_string(&request).expect("Could not serialize QVMRequest");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&json_string).unwrap(),
            serde_json::json!({"type": "multishot", "addresses": {"ro": true}, "trials": 10, "quil-instructions": "H 0"})
        );
    }
}
