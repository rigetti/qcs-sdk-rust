//! This module provides bindings for compiling programs with the Quilc compiler.

use std::collections::HashMap;
use std::convert::TryFrom;
use std::str::FromStr;

use quil_rs::program::{Program, ProgramError};
use serde::{Deserialize, Deserializer, Serialize};

use qcs_api_client_openapi::models::InstructionSetArchitecture;

use super::isa::{self, Compiler};
use super::rpcq;

use crate::qpu::client::Qcs;

/// Number of seconds to wait before timing out.
pub const DEFAULT_COMPILER_TIMEOUT: f64 = 30.0;

/// Take in a Quil program and produce a "native quil" output from quilc
///
/// # Arguments
///
/// * `program`: The Quil program to compile.
/// * `isa`: The [`InstructionSetArchitecture`] of the targeted platform. Get this using
///     [`super::get_isa`].
/// * `timeout`: The number of seconds to wait before timing out. If not set, defaults to [`DEFAULT_COMPILER_TIMEOUT`].
///
/// returns: `eyre::Result<quil_rs::Program>`
///
/// # Errors
///
/// `eyre` is used to create human-readable error messages, since most of the errors are not
/// recoverable at runtime. This function can fail generally if the provided ISA cannot be converted
/// into a form that `quilc` recognizes, if `quilc` cannot be contacted, or if the program cannot
/// be converted by `quilc`.
#[cfg_attr(
    feature = "tracing",
    tracing::instrument(skip(client), level = "trace")
)]
pub fn compile_program(
    quil: &str,
    isa: TargetDevice,
    client: &Qcs,
    options: CompilerOpts,
) -> Result<CompilationResult, Error> {
    #[cfg(feature = "tracing")]
    tracing::debug!(compiler_options=?options, "compiling quil program with quilc",);

    let config = client.get_config();
    let endpoint = config.quilc_url();
    let params = QuilcParams::new(quil, isa).with_protoquil(options.protoquil);
    let request =
        rpcq::RPCRequest::new("quil_to_native_quil", &params).with_timeout(options.timeout);
    let rpcq_client = rpcq::Client::new(endpoint)
        .map_err(|source| Error::from_quilc_error(endpoint.into(), source))?;
    match rpcq_client.run_request::<_, QuilToNativeQuilResponse>(&request) {
        Ok(response) => Ok(CompilationResult {
            program: Program::from_str(&response.quil).map_err(Error::Parse)?,
            native_quil_metadata: response.metadata,
        }),
        Err(source) => Err(Error::from_quilc_error(endpoint.into(), source)),
    }
}

/// The result of compiling a Quil program to native quil with `quilc`
#[derive(Clone, Debug, PartialEq)]
pub struct CompilationResult {
    /// The compiled program
    pub program: Program,
    /// Metadata about the compiled program
    pub native_quil_metadata: Option<NativeQuilMetadata>,
}

/// A set of options that determine the behavior of compiling programs with quilc
#[derive(Clone, Copy, Debug)]
pub struct CompilerOpts {
    /// The number of seconds to wait before timing out. If `None`, there is no timeout.
    timeout: Option<f64>,

    /// If the compiler should produce "protoquil" as output. If `None`, the default
    /// behavior configured in the compiler service is used.
    protoquil: Option<bool>,
}

/// Functions for building a [`CompilerOpts`] instance
impl CompilerOpts {
    /// Creates a new instance of [`CompilerOpts`] with zero values for each option.
    /// Consider using [`CompilerOpts::default()`] to create an instance with recommended defaults.
    #[must_use]
    pub fn new() -> Self {
        Self {
            timeout: None,
            protoquil: None,
        }
    }

    /// Set the number of seconds to wait before timing out. If set to None, the timeout is disabled.
    #[must_use]
    pub fn with_timeout(&mut self, seconds: Option<f64>) -> Self {
        self.timeout = seconds;
        *self
    }

    /// Set to control whether the compiler should produce "protoquil" as output.
    /// If `None`, the default behavior configured in the compiler service is used.
    #[must_use]
    pub fn with_protoquil(&mut self, protoquil: Option<bool>) -> Self {
        self.protoquil = protoquil;
        *self
    }
}

impl Default for CompilerOpts {
    /// Default compiler options
    /// * `timeout`: See [`DEFAULT_COMPILER_TIMEOUT`]
    fn default() -> Self {
        Self {
            timeout: Some(DEFAULT_COMPILER_TIMEOUT),
            protoquil: None,
        }
    }
}

/// Fetch the version information from the running Quilc compiler.
pub fn get_version_info(client: &Qcs) -> Result<String, Error> {
    #[cfg(feature = "tracing")]
    tracing::debug!("requesting quilc version information");

    let config = client.get_config();
    let endpoint = config.quilc_url();
    let binding: HashMap<String, String> = HashMap::new();
    let request = rpcq::RPCRequest::new("get_version_info", &binding);
    let rpcq_client = rpcq::Client::new(endpoint)
        .map_err(|source| Error::from_quilc_error(endpoint.into(), source))?;
    match rpcq_client.run_request::<_, QuilcVersionResponse>(&request) {
        Ok(response) => Ok(response.quilc),
        Err(source) => Err(Error::from_quilc_error(endpoint.into(), source)),
    }
}

/// Pauli Term
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, PartialOrd)]
#[serde(tag = "_type")]
pub struct PauliTerm {
    /// Qubit indices onto which the factors of the Pauli term are applied.
    pub indices: Vec<u64>,

    /// Ordered factors of the Pauli term.
    pub symbols: Vec<String>,
}

/// Request to conjugate a Pauli Term by a Clifford element.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, PartialOrd)]
#[serde(tag = "_type")]
pub struct ConjugateByCliffordRequest {
    /// Pauli Term to conjugate.
    pub pauli: PauliTerm,

    /// Clifford element.
    pub clifford: String,
}

/// The "outer" request shape for a `conjugate_pauli_by_clifford` request.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, PartialOrd)]
struct ConjugatePauliByCliffordRequest {
    #[serde(rename = "*args")]
    args: [ConjugateByCliffordRequest; 1],
}

impl From<ConjugateByCliffordRequest> for ConjugatePauliByCliffordRequest {
    fn from(value: ConjugateByCliffordRequest) -> Self {
        Self { args: [value] }
    }
}

/// Conjugate Pauli by Clifford response.
#[derive(Clone, Deserialize, Debug, PartialEq, PartialOrd)]
pub struct ConjugatePauliByCliffordResponse {
    /// Encoded global phase factor on the emitted Pauli.
    pub phase: i64,

    /// Description of the encoded Pauli.
    pub pauli: String,
}

/// Given a circuit that consists only of elements of the Clifford group,
/// return its action on a `PauliTerm`.
/// In particular, for Clifford ``C``, and Pauli ``P``, this returns the Pauli Term
/// representing ``CPC^{\dagger}``.
pub fn conjugate_pauli_by_clifford(
    client: &Qcs,
    request: ConjugateByCliffordRequest,
) -> Result<ConjugatePauliByCliffordResponse, Error> {
    #[cfg(feature = "tracing")]
    tracing::debug!("requesting quilc conjugate_pauli_by_clifford");

    let config = client.get_config();
    let endpoint = config.quilc_url();
    let request: ConjugatePauliByCliffordRequest = request.into();
    let request = rpcq::RPCRequest::new("conjugate_pauli_by_clifford", &request);
    let rpcq_client = rpcq::Client::new(endpoint)
        .map_err(|source| Error::from_quilc_error(endpoint.into(), source))?;
    match rpcq_client.run_request::<_, ConjugatePauliByCliffordResponse>(&request) {
        Ok(response) => Ok(response),
        Err(source) => Err(Error::from_quilc_error(endpoint.into(), source)),
    }
}

/// Request to generate a randomized benchmarking sequence.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, PartialOrd)]
#[serde(tag = "_type")]
pub struct RandomizedBenchmarkingRequest {
    /// Depth of the benchmarking sequence.
    pub depth: u64,

    /// Number of qubits involved in the benchmarking sequence.
    pub qubits: u64,

    /// List of Quil programs, each describing a Clifford.
    pub gateset: Vec<String>,

    /// PRNG seed. Set this to guarantee repeatable results.
    pub seed: Option<u64>,

    /// Fixed Clifford, specified as a Quil string, to interleave through an RB sequence.
    pub interleaver: Option<String>,
}

/// The "outer" request shape for a `generate_rb_sequence` request.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, PartialOrd)]
struct GenerateRandomizedBenchmarkingSequenceRequest {
    #[serde(rename = "*args")]
    args: [RandomizedBenchmarkingRequest; 1],
}

impl From<RandomizedBenchmarkingRequest> for GenerateRandomizedBenchmarkingSequenceRequest {
    fn from(value: RandomizedBenchmarkingRequest) -> Self {
        Self { args: [value] }
    }
}

/// Randomly generated benchmarking sequence response.
#[derive(Clone, Deserialize, Debug, PartialEq, PartialOrd)]
pub struct GenerateRandomizedBenchmarkingSequenceResponse {
    /// List of Cliffords, each expressed as a list of generator indices.
    pub sequence: Vec<Vec<i64>>,
}

/// Construct a randomized benchmarking experiment on the given qubits, decomposing into
/// gateset. If interleaver is not provided, the returned sequence will have the form
/// ```C_1 C_2 ... C_(depth-1) C_inv ,```
///
/// where each C is a Clifford element drawn from gateset, ``C_{< depth}`` are randomly selected,
/// and ``C_inv`` is selected so that the entire sequence composes to the identity.  If an
/// interleaver ``G`` (which must be a Clifford, and which will be decomposed into the native
/// gateset) is provided, then the sequence instead takes the form
/// ```C_1 G C_2 G ... C_(depth-1) G C_inv .```
pub fn generate_randomized_benchmarking_sequence(
    client: &Qcs,
    request: RandomizedBenchmarkingRequest,
) -> Result<GenerateRandomizedBenchmarkingSequenceResponse, Error> {
    #[cfg(feature = "tracing")]
    tracing::debug!("requesting quilc generate_randomized_benchmarking_sequence");

    let config = client.get_config();
    let endpoint = config.quilc_url();
    let request: GenerateRandomizedBenchmarkingSequenceRequest = request.into();
    let request = rpcq::RPCRequest::new("generate_rb_sequence", &request);
    let rpcq_client = rpcq::Client::new(endpoint)
        .map_err(|source| Error::from_quilc_error(endpoint.into(), source))?;
    match rpcq_client.run_request::<_, GenerateRandomizedBenchmarkingSequenceResponse>(&request) {
        Ok(response) => Ok(response),
        Err(source) => Err(Error::from_quilc_error(endpoint.into(), source)),
    }
}

/// All of the errors that can occur within this module.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An ISA-related error.
    #[error("Problem converting ISA to quilc format. This is a bug in this library or in QCS.")]
    Isa(#[from] isa::Error),
    /// An error when trying to connect to quilc.
    #[error("Problem connecting to quilc at {0}")]
    QuilcConnection(String, #[source] rpcq::Error),
    /// An error when trying to compile using quilc.
    #[error("Problem compiling quil program: {0}")]
    QuilcCompilation(String),
    /// An error when trying to parse the compiled program.
    #[error("Problem when trying to parse the compiled program: {0}")]
    Parse(ProgramError<Program>),
}

impl Error {
    fn from_quilc_error(quilc_uri: String, source: rpcq::Error) -> Self {
        match source {
            rpcq::Error::Response(message) => Error::QuilcCompilation(message),
            source => Error::QuilcConnection(quilc_uri, source),
        }
    }
}

/// The response from quilc for a `quil_to_native_quil` request.
#[derive(Clone, Deserialize, Debug, PartialEq, PartialOrd)]
struct QuilToNativeQuilResponse {
    /// The compiled program
    quil: String,
    /// Metadata about the compiled program
    #[serde(default)]
    metadata: Option<NativeQuilMetadata>,
}

#[allow(unused_qualifications)]
fn deserialize_none_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + std::default::Default,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

/// Metadata about a program compiled to native quil.
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, PartialOrd)]
pub struct NativeQuilMetadata {
    /// Output qubit index relabeling due to SWAP insertion.
    #[serde(deserialize_with = "deserialize_none_as_default")]
    pub final_rewiring: Vec<u64>,
    /// Maximum number of successive gates in the native Quil program.
    pub gate_depth: Option<u64>,
    /// Total number of gates in the native Quil program.
    pub gate_volume: Option<u64>,
    /// Maximum number of two-qubit gates in the native Quil program.
    pub multiqubit_gate_depth: Option<u64>,
    /// Rough estimate of native quil program length in seconds.
    pub program_duration: Option<f64>,
    /// Rough estimate of fidelity of the native Quil program.
    pub program_fidelity: Option<f64>,
    /// Total number of swaps in the native Quil program.
    pub topological_swaps: Option<u64>,
    /// The estimated runtime of the program on a Rigetti QPU, in milliseconds. Available only for
    /// protoquil compliant programs.
    pub qpu_runtime_estimation: Option<f64>,
}

#[derive(Clone, Deserialize, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct QuilcVersionResponse {
    quilc: String,
}

/// The top level params that get passed to quilc
#[derive(Serialize, Debug, Clone, PartialEq)]
struct QuilcParams {
    protoquil: Option<bool>,
    #[serde(rename = "*args")]
    args: [NativeQuilRequest; 1],
}

impl QuilcParams {
    fn new(quil: &str, isa: TargetDevice) -> Self {
        Self {
            protoquil: None,
            args: [NativeQuilRequest::new(quil, isa)],
        }
    }

    fn with_protoquil(self, protoquil: Option<bool>) -> Self {
        Self { protoquil, ..self }
    }
}

/// The expected request structure for sending Quil to quilc to be compiled
#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(tag = "_type")]
struct NativeQuilRequest {
    quil: String,
    target_device: TargetDevice,
}

impl NativeQuilRequest {
    fn new(quil: &str, target_device: TargetDevice) -> Self {
        Self {
            quil: String::from(quil),
            target_device,
        }
    }
}

/// Description of a device to compile for.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "_type")]
pub struct TargetDevice {
    isa: Compiler,
    specs: HashMap<String, String>,
}

impl TryFrom<InstructionSetArchitecture> for TargetDevice {
    type Error = Error;

    fn try_from(isa: InstructionSetArchitecture) -> Result<Self, Self::Error> {
        Ok(Self {
            isa: Compiler::try_from(isa)?,
            specs: HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::qvm::api::AddressRequest;

    use super::*;
    use qcs_api_client_openapi::models::InstructionSetArchitecture;
    use regex::Regex;
    use std::{fs::File, num::NonZeroU16};

    const EXPECTED_H0_OUTPUT: &str = "MEASURE 0\n";

    fn aspen_9_isa() -> InstructionSetArchitecture {
        serde_json::from_reader(File::open("tests/aspen_9_isa.json").unwrap()).unwrap()
    }

    pub(crate) fn qvm_isa() -> InstructionSetArchitecture {
        serde_json::from_reader(File::open("tests/qvm_isa.json").unwrap()).unwrap()
    }

    #[tokio::test]
    async fn compare_native_quil_to_expected_output() {
        let output = compile_program(
            "MEASURE 0",
            TargetDevice::try_from(qvm_isa()).expect("Couldn't build target device from ISA"),
            &Qcs::load().await.unwrap_or_default(),
            CompilerOpts::default(),
        )
        .expect("Could not compile");
        assert_eq!(output.program.to_string(true), EXPECTED_H0_OUTPUT);
    }

    const BELL_STATE: &str = r##"DECLARE ro BIT[2]

H 0
CNOT 0 1

MEASURE 0 ro[0]
MEASURE 1 ro[1]
"##;

    #[tokio::test]
    async fn run_compiled_bell_state_on_qvm() {
        let client = Qcs::load().await.unwrap_or_default();
        let output = compile_program(
            BELL_STATE,
            TargetDevice::try_from(aspen_9_isa()).expect("Couldn't build target device from ISA"),
            &client,
            CompilerOpts::default(),
        )
        .expect("Could not compile");
        let mut results = crate::qvm::Execution::new(&output.program.to_string(true))
            .unwrap()
            .run(
                NonZeroU16::new(10).expect("value is non-zero"),
                [("ro".to_string(), AddressRequest::IncludeAll)]
                    .iter()
                    .cloned()
                    .collect(),
                &HashMap::default(),
                &client.get_config(),
            )
            .await
            .expect("Could not run program on QVM");
        for shot in results
            .memory
            .remove("ro")
            .expect("Did not receive ro buffer")
            .into_i8()
            .unwrap()
        {
            assert_eq!(shot.len(), 2);
            assert_eq!(shot[0], shot[1]);
        }
    }

    #[tokio::test]
    async fn test_compile_declare_only() {
        let client = Qcs::load().await.unwrap_or_default();
        let output = compile_program(
            "DECLARE ro BIT[1]\n",
            TargetDevice::try_from(aspen_9_isa()).expect("Couldn't build target device from ISA"),
            &client,
            CompilerOpts::default(),
        )
        .expect("Should be able to compile");
        assert_eq!(output.program.to_string(true), "DECLARE ro BIT[1]\n");
        assert_ne!(output.native_quil_metadata, None);
    }

    #[tokio::test]
    async fn get_version_info_from_quilc() {
        let client = Qcs::load().await.unwrap_or_default();
        let version = get_version_info(&client).expect("Should get version info from quilc");
        let semver_re = Regex::new(r"^([0-9]+)\.([0-9]+)\.([0-9]+)$").unwrap();
        assert!(semver_re.is_match(&version));
    }

    #[tokio::test]
    async fn test_conjugate_pauli_by_clifford() {
        let client = Qcs::load().await.unwrap_or_default();
        let request = ConjugateByCliffordRequest {
            pauli: PauliTerm {
                indices: vec![0],
                symbols: vec!["X".into()],
            },
            clifford: "H 0".into(),
        };
        let response = conjugate_pauli_by_clifford(&client, request)
            .expect("Should conjugate pauli by clifford");

        assert_eq!(
            response,
            ConjugatePauliByCliffordResponse {
                phase: 0,
                pauli: "Z".into(),
            }
        );
    }

    #[tokio::test]
    async fn test_generate_randomized_benchmark_sequence() {
        let client = Qcs::load().await.unwrap_or_default();
        let request = RandomizedBenchmarkingRequest {
            depth: 2,
            qubits: 1,
            gateset: vec!["X 0", "H 0"].into_iter().map(String::from).collect(),
            seed: Some(314),
            interleaver: Some("Y 0".into()),
        };
        let response = generate_randomized_benchmarking_sequence(&client, request)
            .expect("Should generate randomized benchmark sequence");

        assert_eq!(
            response,
            GenerateRandomizedBenchmarkingSequenceResponse {
                sequence: vec![vec![1, 0], vec![0, 1, 0, 1], vec![1, 0]],
            }
        );
    }
}
