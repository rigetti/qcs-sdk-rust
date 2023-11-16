//! This module provides types and functions for making HTTP-based API calls to the QVM.
//! Consider [`super::run_program`] for higher level access to the QVM that allows
//! for running parameterized programs.
use std::{collections::HashMap, num::NonZeroU16};

use reqwest::Response;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{client::Qcs, RegisterData};

use super::{Error, QvmOptions};

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
    // This bound is required for `T` to be recognized as being `Deserialize`
    // https://github.com/serde-rs/serde/issues/1296
    #[serde(bound = "")]
    Success(T),
    Failure(Failure),
}

impl<T: DeserializeOwned> QvmResponse<T> {
    /// Converts a [`QvmResponse<T>`] into a [`Result<T, qvm::Error>`] containing the successful response,
    /// as the Ok value or an [`Error`] containing the error message from the [`Failure`] response.
    pub(super) fn into_result(self) -> Result<T, Error> {
        match self {
            Self::Success(response) => Ok(response),
            Self::Failure(response) => Err(Error::Qvm {
                message: response.status,
            }),
        }
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
    pub trials: NonZeroU16,
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
    /// Get all values for the address.
    #[serde(serialize_with = "serialize_true")]
    IncludeAll,
    /// Exclude all values for the address.
    #[serde(serialize_with = "serialize_false")]
    ExcludeAll,
    /// A list of specific indices to get back for the address.
    Indices(Vec<usize>),
}

fn serialize_true<S>(serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_bool(true)
}

fn serialize_false<S>(serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_bool(false)
}

impl MultishotRequest {
    /// Creates a new [`MultishotRequest`] with the given parameters.
    #[must_use]
    pub fn new(
        compiled_quil: String,
        trials: NonZeroU16,
        addresses: HashMap<String, AddressRequest>,
        measurement_noise: Option<(f64, f64, f64)>,
        gate_noise: Option<(f64, f64, f64)>,
        rng_seed: Option<i64>,
    ) -> Self {
        Self {
            compiled_quil,
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

/// The request body needed for a [`run_and_measure`] request to the QVM.
#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct MultishotMeasureRequest {
    /// The Quil program to run.
    pub compiled_quil: String,
    /// The number of trials ("shots") to run the program.
    pub trials: NonZeroU16,
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
        compiled_quil: String,
        trials: NonZeroU16,
        qubits: &[u64],
        measurement_noise: Option<(f64, f64, f64)>,
        gate_noise: Option<(f64, f64, f64)>,
        rng_seed: Option<i64>,
    ) -> Self {
        Self {
            compiled_quil,
            trials,
            qubits: qubits.to_vec(),
            measurement_noise,
            gate_noise,
            rng_seed,
            request_type: RequestType::MultishotMeasure,
        }
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

/// Provides HTTP-based access to QVM functionality
#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct HttpClient {
    client: reqwest::Client,
    /// Address used to connect to the QVM
    pub qvm_url: String,
}

impl HttpClient {
    /// Build a new [`HttpClient`] to connect to a QVM server at `qvm_url`.
    #[must_use]
    pub fn new(qvm_url: String) -> Self {
        let client = reqwest::Client::new();
        Self { client, qvm_url }
    }
}

impl From<&Qcs> for HttpClient {
    fn from(qcs: &Qcs) -> Self {
        Self::new(qcs.get_config().qvm_url().to_string())
    }
}

#[async_trait::async_trait]
impl super::Client for HttpClient {
    async fn get_version_info(&self, options: &QvmOptions) -> Result<String, Error> {
        #[cfg(feature = "tracing")]
        tracing::debug!("requesting qvm version information");
        let params = HashMap::from([("type", "version")]);
        let response = make_request(&params, self, options).await?;
        let qvm_url = self.qvm_url.clone();
        if response.status() == 200 {
            response
                .text()
                .await
                .map_err(|source| Error::QvmCommunication { qvm_url, source })
        } else {
            match response.json::<Failure>().await {
                Ok(Failure { status: message }) => Err(Error::Qvm { message }),
                Err(source) => Err(Error::QvmCommunication { qvm_url, source }),
            }
        }
    }

    async fn run(
        &self,
        request: &MultishotRequest,
        options: &QvmOptions,
    ) -> Result<MultishotResponse, Error> {
        #[cfg(feature = "tracing")]
        tracing::debug!("making a multishot request to the QVM");
        let response = make_request(request, self, options).await?;
        response
            .json::<QvmResponse<MultishotResponse>>()
            .await
            .map(QvmResponse::into_result)
            .map_err(|source| Error::QvmCommunication {
                qvm_url: self.qvm_url.clone(),
                source,
            })?
    }

    async fn run_and_measure(
        &self,
        request: &MultishotMeasureRequest,
        options: &QvmOptions,
    ) -> Result<Vec<Vec<i64>>, Error> {
        let response = make_request(request, self, options).await?;
        response
            .json::<QvmResponse<Vec<Vec<i64>>>>()
            .await
            .map(QvmResponse::into_result)
            .map_err(|source| Error::QvmCommunication {
                qvm_url: self.qvm_url.clone(),
                source,
            })?
    }

    async fn measure_expectation(
        &self,
        request: &ExpectationRequest,
        options: &QvmOptions,
    ) -> Result<Vec<f64>, Error> {
        let response = make_request(request, self, options).await?;
        response
            .json::<QvmResponse<Vec<f64>>>()
            .await
            .map(QvmResponse::into_result)
            .map_err(|source| Error::QvmCommunication {
                qvm_url: self.qvm_url.clone(),
                source,
            })?
    }

    async fn get_wavefunction(
        &self,
        request: &WavefunctionRequest,
        options: &QvmOptions,
    ) -> Result<Vec<u8>, Error> {
        let response = make_request(request, self, options).await?;
        if response.status() == 200 {
            response
                .bytes()
                .await
                .map(Into::into)
                .map_err(|source| Error::QvmCommunication {
                    qvm_url: self.qvm_url.clone(),
                    source,
                })
        } else {
            match response.json::<Failure>().await {
                Ok(Failure { status: message }) => Err(Error::Qvm { message }),
                Err(source) => Err(Error::QvmCommunication {
                    qvm_url: self.qvm_url.clone(),
                    source,
                }),
            }
        }
    }
}

async fn make_request<T>(
    request: &T,
    client: &HttpClient,
    options: &QvmOptions,
) -> Result<Response, Error>
where
    T: Serialize,
{
    let mut post = client.client.post(&client.qvm_url).json(request);
    if let Some(timeout) = options.timeout {
        post = post.timeout(timeout);
    }
    post.send().await.map_err(|source| Error::QvmCommunication {
        qvm_url: client.qvm_url.clone(),
        source,
    })
}

#[cfg(test)]
mod describe_request {
    use std::{collections::HashMap, num::NonZeroU16};

    use crate::qvm::http::AddressRequest;

    use super::MultishotRequest;

    #[test]
    fn it_includes_the_program() {
        let program = "H 0";
        let request = MultishotRequest::new(
            program.to_string(),
            NonZeroU16::new(1).expect("value is non-zero"),
            HashMap::new(),
            None,
            None,
            None,
        );
        assert_eq!(&request.compiled_quil, program);
    }

    #[test]
    fn it_uses_kebab_case_for_json() {
        let request = MultishotRequest::new(
            "H 0".to_string(),
            NonZeroU16::new(10).expect("value is non-zero"),
            [("ro".to_string(), AddressRequest::IncludeAll)]
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
