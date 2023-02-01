//! This module contains all the functionality for running Quil programs on a QVM. Specifically,
//! the [`Execution`] struct in this module.

use std::{borrow::Cow, collections::HashMap};

use serde::{Deserialize, Serialize};

pub(crate) use execution::{Error, Execution};

use crate::RegisterData;

mod execution;

/// Encapsulates data returned after running a program on the QVM
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct QvmResultData {
    /// A map of register names (ie. "ro") to a [`RegisterData`] containing their values.
    pub memory: HashMap<String, RegisterData>,
}

impl QvmResultData {
    pub(crate) fn from_qvm_response(register_data_map: HashMap<String, RegisterData>) -> Self {
        Self {
            memory: register_data_map,
        }
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub(super) enum Response {
    Success(Success),
    Failure(Failure),
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub(super) struct Success {
    #[serde(flatten)]
    pub(super) registers: HashMap<String, RegisterData>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub(super) struct Failure {
    /// The message from QVM describing what went wrong.
    pub(super) status: String,
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
    fn new(program: &str, shots: u16, readouts: &'request [Cow<'request, str>]) -> Self {
        let addresses: HashMap<&str, bool> = readouts.iter().map(|v| (v.as_ref(), true)).collect();
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
    use std::borrow::Cow;

    use super::Request;

    #[test]
    fn it_includes_the_program() {
        let program = "H 0";
        let request = Request::new(program, 1, &[]);
        assert_eq!(&request.quil_instructions, program);
    }

    #[test]
    fn it_uses_kebab_case_for_json() {
        let request = Request::new("H 0", 10, &[Cow::Borrowed("ro")]);
        let json_string = serde_json::to_string(&request).expect("Could not serialize QVMRequest");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&json_string).unwrap(),
            serde_json::json!({"type": "multishot", "addresses": {"ro": true}, "trials": 10, "quil-instructions": "H 0"})
        );
    }
}
