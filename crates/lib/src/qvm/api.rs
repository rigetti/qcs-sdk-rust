//! This module provides types and functions for making API calls directly to the QVM.
//! Consider [`super::run_program`] for higher level access to the QVM that allows
//! for running parameterized programs.
use std::collections::HashMap;

use qcs_api_client_common::ClientConfiguration;
use reqwest::Response;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::RegisterData;

use super::Error;

#[derive(Serialize, Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum RequestType {
    Multishot,
    MultishotMeasure,
    Expectation,
    Wavefunction,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub(super) struct Failure {
    /// The message from QVM describing what went wrong.
    pub(super) status: String,
}

/// A QVM response that can be deserialized into some successful response type `T`, or into a
/// [`Failure`] containing an error message.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub(super) enum QvmResponse<T>
where
    T: DeserializeOwned,
{
    #[serde(bound = "")]
    Success(T),
    Failure(Failure),
}

/// Fetch the version information from the running QVM server.
pub async fn get_version_info(config: &ClientConfiguration) -> Result<String, Error> {
    #[cfg(feature = "tracing")]
    tracing::debug!("requesting qvm version information");
    let params = HashMap::from([("type", "version")]);
    let response = make_request(&params, config).await?;
    if response.status() == 200 {
        response
            .text()
            .await
            .map_err(|source| Error::QvmCommunication {
                qvm_url: config.qvm_url().into(),
                source,
            })
    } else {
        match response.json::<Failure>().await {
            Ok(Failure { status: message }) => Err(Error::Qvm { message }),
            Err(source) => Err(Error::QvmCommunication {
                qvm_url: config.qvm_url().into(),
                source,
            }),
        }
    }
}

/// Executes a program on the QVM.
pub async fn run(
    request: &MultishotRequest,
    config: &ClientConfiguration,
) -> Result<MultishotResponse, Error> {
    #[cfg(feature = "tracing")]
    tracing::debug!("making a multishot request to the QVM");
    let response = make_request(request, config).await?;
    match response.json::<QvmResponse<MultishotResponse>>().await {
        Ok(QvmResponse::Success(response)) => Ok(response),
        Ok(QvmResponse::Failure(response)) => Err(Error::Qvm {
            message: response.status,
        }),
        Err(source) => Err(Error::QvmCommunication {
            qvm_url: config.qvm_url().into(),
            source,
        }),
    }
}

/// The request body needed to make a multishot [`run`] request to the QVM.
#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct MultishotRequest {
    /// The Quil program to run.
    pub compiled_quil: String,
    /// The memory regions to include in the response.
    pub addresses: HashMap<String, AddressRequest>,
    /// The number of trials ("shots") to run.
    pub trials: u16,
    /// Simulated measurement noise for the X, Y, and Z axes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub measurement_noise: Option<(f64, f64, f64)>,
    /// Simulated gate noise for the X, Y, and Z axes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gate_noise: Option<(f64, f64, f64)>,
    /// An optional seed for the random number generator.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rng_seed: Option<i64>,
    #[serde(rename = "type")]
    request_type: RequestType,
}

/// An enum encapsulating the different ways to request data back from the QVM for an address.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum AddressRequest {
    /// Set to `true` to request all values at this address, or `false` to explicitly exclude
    /// the address.
    All(bool),
    /// A list of specific indices to get back for the address.
    Indices(Vec<usize>),
}

impl MultishotRequest {
    /// Creates a new [`MultishotRequest`] with the given parameters.
    #[must_use]
    pub fn new(
        program: String,
        trials: u16,
        addresses: HashMap<String, AddressRequest>,
        measurement_noise: Option<(f64, f64, f64)>,
        gate_noise: Option<(f64, f64, f64)>,
        rng_seed: Option<i64>,
    ) -> Self {
        Self {
            compiled_quil: program,
            addresses,
            trials,
            measurement_noise,
            gate_noise,
            rng_seed,
            request_type: RequestType::Multishot,
        }
    }
}

/// The response body returned by the QVM after a multishot [`run`] request.
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct MultishotResponse {
    /// The requested readout registers and their final values for each shot.
    #[serde(flatten)]
    pub registers: HashMap<String, RegisterData>,
}

/// Executes a program on the QVM, measuring and returning the state of the qubits at the end of each trial.
pub async fn run_and_measure(
    request: &MultishotMeasureRequest,
    config: &ClientConfiguration,
) -> Result<Vec<Vec<i64>>, Error> {
    let response = make_request(request, config).await?;
    match response.json::<QvmResponse<Vec<Vec<i64>>>>().await {
        Ok(QvmResponse::Success(response)) => Ok(response),
        Ok(QvmResponse::Failure(response)) => Err(Error::Qvm {
            message: response.status,
        }),
        Err(source) => Err(Error::QvmCommunication {
            qvm_url: config.qvm_url().into(),
            source,
        }),
    }
}

/// The request body needed for a [`run_and_measure_program`] request to the QVM.
#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct MultishotMeasureRequest {
    /// The Quil program to run.
    pub compiled_quil: String,
    /// The number of trials ("shots") to run the program.
    pub trials: u16,
    /// Qubits to measure
    pub qubits: Vec<u64>,
    /// Simulated measurement noise for the X, Y, and Z axes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub measurement_noise: Option<(f64, f64, f64)>,
    /// Simulated gate noise for the X, Y, and Z axes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gate_noise: Option<(f64, f64, f64)>,
    /// An optional seed for the random number generator.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rng_seed: Option<i64>,
    #[serde(rename = "type")]
    request_type: RequestType,
}

impl MultishotMeasureRequest {
    /// Construct a new [`MultishotMeasureRequest`] using the given parameters.
    #[must_use]
    pub fn new(
        program: String,
        shots: u16,
        qubits: &[u64],
        measurement_noise: Option<(f64, f64, f64)>,
        gate_noise: Option<(f64, f64, f64)>,
        rng_seed: Option<i64>,
    ) -> Self {
        Self {
            compiled_quil: program,
            trials: shots,
            qubits: qubits.to_vec(),
            measurement_noise,
            gate_noise,
            rng_seed,
            request_type: RequestType::MultishotMeasure,
        }
    }
}

/// Measure the expectation value of pauli operators given a defined state.
pub async fn measure_expectation(
    request: &ExpectationRequest,
    config: &ClientConfiguration,
) -> Result<Vec<f64>, Error> {
    let response = make_request(request, config).await?;
    match response.json::<QvmResponse<Vec<f64>>>().await {
        Ok(QvmResponse::Success(response)) => Ok(response),
        Ok(QvmResponse::Failure(response)) => Err(Error::Qvm {
            message: response.status,
        }),
        Err(source) => Err(Error::QvmCommunication {
            qvm_url: config.qvm_url().into(),
            source,
        }),
    }
}

/// The request body needed for a [`measure_expectation`] request to the QVM.
#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct ExpectationRequest {
    /// A Quil program defining the state.
    pub state_preparation: String,
    /// A list of Pauli operators to measure.
    pub operators: Vec<String>,
    /// An optional seed for the random number generator.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rng_seed: Option<i64>,
    #[serde(rename = "type")]
    request_type: RequestType,
}

impl ExpectationRequest {
    /// Creates a new [`ExpectationRequest`] using the given parameters.
    #[must_use]
    pub fn new(state_preparation: String, operators: &[String], rng_seed: Option<i64>) -> Self {
        Self {
            state_preparation,
            operators: operators.to_vec(),
            rng_seed,
            request_type: RequestType::Expectation,
        }
    }
}

/// Run a program and retrieve the resulting wavefunction.
pub async fn get_wavefunction(
    request: &WavefunctionRequest,
    config: &ClientConfiguration,
) -> Result<Vec<u8>, Error> {
    let response = make_request(request, config).await?;
    if response.status() == 200 {
        response
            .bytes()
            .await
            .map(Into::into)
            .map_err(|source| Error::QvmCommunication {
                qvm_url: config.qvm_url().into(),
                source,
            })
    } else {
        match response.json::<Failure>().await {
            Ok(Failure { status: message }) => Err(Error::Qvm { message }),
            Err(source) => Err(Error::QvmCommunication {
                qvm_url: config.qvm_url().into(),
                source,
            }),
        }
    }
}

/// The request body needed to make a [`get_wavefunction`] request to the QVM.
#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct WavefunctionRequest {
    /// The Quil program to run.
    pub compiled_quil: String,
    /// Simulated measurement noise for the X, Y, and Z axes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub measurement_noise: Option<(f64, f64, f64)>,
    /// Simulated gate noise for the X, Y, and Z axes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gate_noise: Option<(f64, f64, f64)>,
    /// An optional seed for the random number generator.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rng_seed: Option<i64>,
    #[serde(rename = "type")]
    request_type: RequestType,
}

impl WavefunctionRequest {
    /// Create a new [`WavefunctionRequest`] with the given parameters.
    #[must_use]
    pub fn new(
        compiled_quil: String,
        measurement_noise: Option<(f64, f64, f64)>,
        gate_noise: Option<(f64, f64, f64)>,
        rng_seed: Option<i64>,
    ) -> Self {
        Self {
            compiled_quil,
            measurement_noise,
            gate_noise,
            rng_seed,
            request_type: RequestType::Wavefunction,
        }
    }
}

async fn make_request<T>(request: &T, config: &ClientConfiguration) -> Result<Response, Error>
where
    T: Serialize,
{
    let client = reqwest::Client::new();
    client
        .post(config.qvm_url())
        .json(request)
        .send()
        .await
        .map_err(|source| Error::QvmCommunication {
            qvm_url: config.qvm_url().into(),
            source,
        })
}

#[cfg(test)]
mod describe_request {
    use std::collections::HashMap;

    use crate::qvm::api::AddressRequest;

    use super::MultishotRequest;

    #[test]
    fn it_includes_the_program() {
        let program = "H 0";
        let request =
            MultishotRequest::new(program.to_string(), 1, HashMap::new(), None, None, None);
        assert_eq!(&request.compiled_quil, program);
    }

    #[test]
    fn it_uses_kebab_case_for_json() {
        let request = MultishotRequest::new(
            "H 0".to_string(),
            10,
            [("ro".to_string(), AddressRequest::All(true))]
                .iter()
                .cloned()
                .collect(),
            Some((1.0, 2.0, 3.0)),
            Some((3.0, 2.0, 1.0)),
            Some(100),
        );
        let json_string = serde_json::to_string(&request).expect("Could not serialize QVMRequest");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&json_string).unwrap(),
            serde_json::json!({"type": "multishot", "addresses": {"ro": true}, "trials": 10, "compiled-quil": "H 0", "measurement-noise": [1.0, 2.0, 3.0], "gate-noise": [3.0, 2.0, 1.0], "rng-seed": 100})
        );
    }
}
