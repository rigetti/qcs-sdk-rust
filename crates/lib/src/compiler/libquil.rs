//! Uses [`libquil-sys`] as the client for Quilc

use std::convert::TryInto;
use std::ffi::NulError;
use std::num::TryFromIntError;
use std::str::FromStr;
use std::string::String;
use std::{convert::TryFrom, ffi::CString};

use super::quilc::{self, NativeQuilMetadata};

/// The errors that can arise when using libquil as a QVM client
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error when calling [`libquil_sys::quilc`]
    #[error("error when calling libquil_sys: {0}")]
    Quilc(#[from] libquil_sys::quilc::Error),
    /// Error when serializing a program
    #[error("error when serializing program: {0}")]
    SerializeProgram(#[from] serde_json::Error),
    /// Error when parsing a program
    #[error("error when parsing program: {0}")]
    ParseProgram(#[from] quil_rs::program::ProgramError),
    /// Error when casting u64 to u32
    #[error("error when casting u64 to u32: {0}")]
    U64Truncation(#[from] TryFromIntError),
    /// Error when creating a [`CString`]
    #[error("error when creating CString: {0}")]
    CString(#[from] NulError),
}

impl From<Error> for quilc::Error {
    fn from(error: Error) -> Self {
        quilc::Error::QuilcCompilation(quilc::CompilationError::Libquil(error))
    }
}

impl From<libquil_sys::quilc::CompilationMetadata> for NativeQuilMetadata {
    fn from(value: libquil_sys::quilc::CompilationMetadata) -> Self {
        NativeQuilMetadata {
            final_rewiring: value.final_rewiring.iter().map(|r| u64::from(*r)).collect(),
            gate_depth: value.gate_depth.map(u64::from),
            gate_volume: value.gate_volume.map(u64::from),
            multiqubit_gate_depth: value.multiqubit_gate_depth.map(u64::from),
            program_duration: value.program_duration,
            program_fidelity: value.program_fidelity,
            topological_swaps: value.topological_swaps.map(u64::from),
            qpu_runtime_estimation: value.qpu_runtime_estimation,
        }
    }
}

/// A libquil client providing Quilc functionality
#[derive(Debug, Clone, Copy)]
pub struct Client;

impl quilc::Client for Client {
    fn compile_program(
        &self,
        quil: &str,
        isa: quilc::TargetDevice,
        options: quilc::CompilerOpts,
    ) -> Result<quilc::CompilationResult, quilc::Error> {
        let program = libquil_sys::quilc::Program::from_str(quil).map_err(Error::from)?;
        let isa = serde_json::to_string(&isa).map_err(Error::from)?;
        let chip = libquil_sys::quilc::Chip::from_str(&isa).map_err(Error::from)?;

        let compilation_result = if options.protoquil.unwrap_or(false) {
            libquil_sys::quilc::compile_protoquil(&program, &chip)
        } else {
            libquil_sys::quilc::compile_program(&program, &chip)
        }
        .map_err(Error::from)?;

        let program = compilation_result
            .program
            .to_string()
            .map_err(Error::from)?
            .parse()
            .map_err(Error::from)?;
        Ok(quilc::CompilationResult {
            program,
            native_quil_metadata: compilation_result.metadata.map(Into::into),
        })
    }

    fn get_version_info(&self) -> Result<String, quilc::Error> {
        libquil_sys::quilc::get_version_info()
            .map(|info| info.version)
            .map_err(|e| Error::from(e).into())
    }

    fn conjugate_pauli_by_clifford(
        &self,
        request: quilc::ConjugateByCliffordRequest,
    ) -> Result<quilc::ConjugatePauliByCliffordResponse, quilc::Error> {
        let pauli_terms = request
            .pauli
            .symbols
            .into_iter()
            .map(CString::new)
            .collect::<Result<_, _>>()
            .map_err(Error::from)?;
        let result = libquil_sys::quilc::conjugate_pauli_by_clifford(
            request
                .pauli
                .indices
                .into_iter()
                .map(u32::try_from)
                .collect::<Result<_, _>>()
                .map_err(Error::from)?,
            pauli_terms,
            &request.clifford.parse().map_err(Error::from)?,
        )
        .map_err(Error::from)?;
        Ok(quilc::ConjugatePauliByCliffordResponse {
            phase: i64::from(result.phase),
            pauli: result.pauli,
        })
    }

    fn generate_randomized_benchmarking_sequence(
        &self,
        request: quilc::RandomizedBenchmarkingRequest,
    ) -> Result<quilc::GenerateRandomizedBenchmarkingSequenceResponse, quilc::Error> {
        let gateset = request
            .gateset
            .iter()
            .map(String::as_str)
            .map(str::parse)
            .collect::<Result<Vec<_>, _>>()
            .map_err(Error::from)?;
        let gateset = gateset.iter().collect();
        let interleaver = request
            .interleaver
            .map(|s| s.parse::<libquil_sys::quilc::Program>())
            .transpose()
            .map_err(Error::from)?;
        let seed = request
            .seed
            .map(i32::try_from)
            .transpose()
            .map_err(Error::from)?;
        let result = libquil_sys::quilc::generate_rb_sequence(
            request.depth.try_into().map_err(Error::from)?,
            request.qubits.try_into().map_err(Error::from)?,
            gateset,
            seed,
            interleaver.as_ref(),
        )
        .map_err(Error::from)?;
        Ok(quilc::GenerateRandomizedBenchmarkingSequenceResponse {
            sequence: result
                .into_iter()
                .map(|i| i.into_iter().map(Into::into).collect())
                .collect(),
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{
        compiler::quilc::{
            Client as _, CompilerOpts, ConjugateByCliffordRequest,
            ConjugatePauliByCliffordResponse, GenerateRandomizedBenchmarkingSequenceResponse,
            PauliTerm, RandomizedBenchmarkingRequest, TargetDevice,
        },
        qvm::{self, http::AddressRequest},
    };

    use super::*;

    use qcs_api_client_openapi::models::InstructionSetArchitecture;
    use quil_rs::quil::Quil;
    use regex::Regex;
    use std::{collections::HashMap, fs::File, num::NonZeroU16};

    const EXPECTED_H0_OUTPUT: &str = "MEASURE 0\n";

    fn aspen_9_isa() -> InstructionSetArchitecture {
        serde_json::from_reader(File::open("tests/aspen_9_isa.json").unwrap()).unwrap()
    }

    pub(crate) fn qvm_isa() -> InstructionSetArchitecture {
        serde_json::from_reader(File::open("tests/qvm_isa.json").unwrap()).unwrap()
    }

    fn quilc_client() -> Client {
        Client {}
    }

    #[tokio::test]
    async fn compare_native_quil_to_expected_output() {
        let output = quilc_client()
            .compile_program(
                "MEASURE 0",
                TargetDevice::try_from(qvm_isa()).expect("Couldn't build target device from ISA"),
                CompilerOpts::default().with_protoquil(Some(true)),
            )
            .expect("Could not compile");
        assert_eq!(output.program.to_quil_or_debug(), EXPECTED_H0_OUTPUT);
    }

    const BELL_STATE: &str = r"DECLARE ro BIT[2]

H 0
CNOT 0 1

MEASURE 0 ro[0]
MEASURE 1 ro[1]
";

    #[tokio::test]
    async fn test_print_isa() {
        let isa = TargetDevice::try_from(aspen_9_isa()).unwrap();
        serde_json::to_string_pretty(&isa).unwrap();
    }

    #[tokio::test]
    async fn run_compiled_bell_state_on_qvm() {
        let qvm_client = qvm::libquil::Client {};
        let output = quilc_client()
            .compile_program(
                BELL_STATE,
                TargetDevice::try_from(aspen_9_isa())
                    .expect("Couldn't build target device from ISA"),
                CompilerOpts::default(),
            )
            .expect("Could not compile");
        let mut results = qvm::Execution::new(&output.program.to_quil_or_debug())
            .unwrap()
            .run(
                NonZeroU16::new(10).expect("value is non-zero"),
                [("ro".to_string(), AddressRequest::IncludeAll)]
                    .iter()
                    .cloned()
                    .collect(),
                &HashMap::default(),
                &qvm_client,
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
        let output = quilc_client()
            .compile_program(
                "DECLARE ro BIT[1]\n",
                TargetDevice::try_from(aspen_9_isa())
                    .expect("Couldn't build target device from ISA"),
                CompilerOpts::default().with_protoquil(Some(true)),
            )
            .expect("Should be able to compile");
        assert_eq!(output.program.to_quil_or_debug(), "DECLARE ro BIT[1]\n");
        assert_ne!(output.native_quil_metadata, None);
    }

    #[tokio::test]
    async fn get_version_info_from_quilc() {
        let rpcq_client = quilc_client();
        let version = rpcq_client
            .get_version_info()
            .expect("Should get version info from quilc");
        let semver_re = Regex::new(r"^([0-9]+)\.([0-9]+)\.([0-9]+)$").unwrap();
        assert!(semver_re.is_match(&version));
    }

    #[tokio::test]
    async fn test_conjugate_pauli_by_clifford() {
        let rpcq_client = quilc_client();
        let request = ConjugateByCliffordRequest {
            pauli: PauliTerm {
                indices: vec![0],
                symbols: vec!["X".into()],
            },
            clifford: "H 0".into(),
        };
        let response = rpcq_client
            .conjugate_pauli_by_clifford(request)
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
    async fn test_conjugate_pauli_by_clifford_2() {
        let rpcq_client = quilc_client();
        let request = ConjugateByCliffordRequest {
            pauli: PauliTerm {
                indices: vec![0],
                symbols: vec!["X".into()],
            },
            clifford: "H 0".into(),
        };
        let response = rpcq_client
            .conjugate_pauli_by_clifford(request)
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
        let rpcq_client = quilc_client();
        let request = RandomizedBenchmarkingRequest {
            depth: 2,
            qubits: 1,
            gateset: vec!["X 0", "H 0"].into_iter().map(String::from).collect(),
            seed: Some(314),
            interleaver: Some("Y 0".into()),
        };
        let response = rpcq_client
            .generate_randomized_benchmarking_sequence(request)
            .expect("Should generate randomized benchmark sequence");

        assert_eq!(
            response,
            GenerateRandomizedBenchmarkingSequenceResponse {
                sequence: vec![vec![1, 0], vec![0, 1, 0, 1], vec![1, 0]],
            }
        );
    }
}
