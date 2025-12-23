use rigetti_pyo3::{create_init_submodule, impl_repr, py_function_sync_async};
use std::convert::TryFrom;

use pyo3::prelude::*;

#[cfg(feature = "stubs")]
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyfunction, gen_stub_pymethods};

use quil_rs::quil::Quil;

use crate::{
    compiler::{
        quilc::{
            self, CompilerOpts, ConjugateByCliffordRequest, ConjugatePauliByCliffordResponse,
            Error, GenerateRandomizedBenchmarkingSequenceResponse, NativeQuilMetadata, PauliTerm,
            RandomizedBenchmarkingRequest, TargetDevice, DEFAULT_COMPILER_TIMEOUT,
        },
        rpcq,
    },
    python::errors,
    qpu::isa::python::PyInstructionSetArchitecture,
};

// compiler
create_init_submodule! {
    submodules: [ "quilc": pyquilc::init_submodule ],
}

mod pyquilc {
    use super::*;
    use rigetti_pyo3::create_init_submodule;

    create_init_submodule! {
        classes: [
            CompilerOpts,
            CompilationResult,
            NativeQuilMetadata,
            TargetDevice,
            PauliTerm,
            ConjugateByCliffordRequest,
            ConjugatePauliByCliffordResponse,
            RandomizedBenchmarkingRequest,
            GenerateRandomizedBenchmarkingSequenceResponse,
            PyQuilcClient
        ],
        consts: [ DEFAULT_COMPILER_TIMEOUT ],
        errors: [ errors::QuilcError ],
        funcs: [
            py_compile_program,
            py_compile_program_async,
            py_get_version_info,
            py_get_version_info_async,
            py_conjugate_pauli_by_clifford,
            py_conjugate_pauli_by_clifford_async,
            py_generate_randomized_benchmarking_sequence,
            py_generate_randomized_benchmarking_sequence_async
        ],
    }
}

impl_repr!(NativeQuilMetadata);

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl CompilerOpts {
    /// Creates a new instance of [`CompilerOpts`] with zero values for each option.
    ///
    /// Consider using [`CompilerOpts::default()`] to create an instance with recommended defaults.
    #[new]
    #[pyo3(signature = (timeout = Some(DEFAULT_COMPILER_TIMEOUT), protoquil = None))]
    fn __new__(timeout: Option<f64>, protoquil: Option<bool>) -> Self {
        Self::new().with_timeout(timeout).with_protoquil(protoquil)
    }

    #[staticmethod]
    #[pyo3(name = "default")]
    fn py_default() -> Self {
        Self::default()
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl TargetDevice {
    #[staticmethod]
    pub fn from_isa(isa: PyInstructionSetArchitecture) -> Result<Self, Error> {
        TargetDevice::try_from(isa.0)
    }

    #[staticmethod]
    pub fn from_json(value: String) -> PyResult<Self> {
        serde_json::from_str(&value).map_err(|err| errors::QuilcError::new_err(err.to_string()))
    }
}

#[derive(Clone, Debug)]
pub(crate) enum QuilcClient {
    Rpcq(rpcq::Client),
    #[cfg(feature = "libquil")]
    LibquilSys(super::libquil::Client),
}

#[derive(Clone)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(module = "qcs_sdk.compiler.quilc", name = "QuilcClient")]
pub(crate) struct PyQuilcClient {
    pub inner: QuilcClient,
}

impl QuilcClient {
    pub(crate) fn as_client(&self) -> &dyn quilc::Client {
        match self {
            QuilcClient::Rpcq(client) => client,
            #[cfg(feature = "libquil")]
            QuilcClient::LibquilSys(client) => client,
        }
    }
}

impl quilc::Client for QuilcClient {
    fn compile_program(
        &self,
        quil: &str,
        isa: TargetDevice,
        options: CompilerOpts,
    ) -> Result<quilc::CompilationResult, Error> {
        self.as_client().compile_program(quil, isa, options)
    }

    fn get_version_info(&self) -> Result<String, Error> {
        self.as_client().get_version_info()
    }

    fn conjugate_pauli_by_clifford(
        &self,
        request: ConjugateByCliffordRequest,
    ) -> Result<ConjugatePauliByCliffordResponse, Error> {
        self.as_client().conjugate_pauli_by_clifford(request)
    }

    fn generate_randomized_benchmarking_sequence(
        &self,
        request: RandomizedBenchmarkingRequest,
    ) -> Result<GenerateRandomizedBenchmarkingSequenceResponse, Error> {
        self.as_client()
            .generate_randomized_benchmarking_sequence(request)
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyQuilcClient {
    #[new]
    fn __new__() -> PyResult<Self> {
        Err(errors::QuilcError::new_err(
            #[cfg(not(feature = "libquil"))]
            "QuilcClient cannot not be instantiated directly. Use QuilcClient.new_rpcq() instead.",
            #[cfg(feature = "libquil")]
            "QuilcClient cannot not be instantiated directly. Use QuilcClient.new_rpcq() or QuilcClient.new_libquil() instead.",
        ))
    }

    #[staticmethod]
    fn new_rpcq(endpoint: &str) -> PyResult<Self> {
        Ok(Self {
            inner: QuilcClient::Rpcq(rpcq::Client::new(endpoint)?),
        })
    }
}

// These are pulled out separately so that the feature flag won't confuse the stub generator.
#[cfg(feature = "libquil")]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyQuilcClient {
    #[staticmethod]
    fn new_libquil() -> Self {
        let libquil_client = qcs::compiler::libquil::Client {};
        Self {
            inner: QuilcClient::LibquilSys(libquil_client),
        }
    }
}

#[cfg(not(feature = "libquil"))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyQuilcClient {
    #[staticmethod]
    #[pyo3(warn(
        message = "The installed version of qcs_sdk was built without the libquil feature. Use QuilcClient.new_rpcq() instead.",
    ))]
    fn new_libquil() -> PyResult<Self> {
        Err(errors::QuilcError::new_err(
            "Cannot create a liquil QuilcClient. Use QuilcClient.new_rpcq() instead.",
        ))
    }
}

py_function_sync_async! {
    #[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk.compiler.quilc"))]
    #[pyfunction]
    #[pyo3(signature = (quil, target, client, options = None))]
    #[tracing::instrument(skip_all)]
    #[pyo3_opentelemetry::pypropagate(on_context_extraction_failure="ignore")]
    async fn compile_program(
        quil: String,
        target: TargetDevice,
        client: PyQuilcClient,
        options: Option<CompilerOpts>,
    ) -> PyResult<CompilationResult> {
        let client = client.inner.as_client();
        let options = options.unwrap_or_default();
        client.compile_program(&quil, target, options)
            .map(|result| CompilationResult {
                program: result
                    .program
                    .to_quil()
                    .expect("successfully compiled program should convert to valid quil"),
                native_quil_metadata: result.native_quil_metadata
            })
            .map_err(Into::into)
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl NativeQuilMetadata {
    #[new]
    #[expect(clippy::too_many_arguments)]
    pub fn __new__(
        final_rewiring: Option<Vec<u64>>,
        gate_depth: Option<u64>,
        gate_volume: Option<u64>,
        multiqubit_gate_depth: Option<u64>,
        program_duration: Option<f64>,
        program_fidelity: Option<f64>,
        topological_swaps: Option<u64>,
        qpu_runtime_estimation: Option<f64>,
    ) -> NativeQuilMetadata {
        Self {
            final_rewiring: final_rewiring.unwrap_or_default(),
            gate_depth,
            gate_volume,
            multiqubit_gate_depth,
            program_duration,
            program_fidelity,
            topological_swaps,
            qpu_runtime_estimation,
        }
    }

    #[expect(clippy::type_complexity)]
    fn __getnewargs__(
        &self,
    ) -> (
        Option<Vec<u64>>,
        Option<u64>,
        Option<u64>,
        Option<u64>,
        Option<f64>,
        Option<f64>,
        Option<u64>,
        Option<f64>,
    ) {
        (
            if self.final_rewiring.is_empty() {
                None
            } else {
                Some(self.final_rewiring.clone())
            },
            self.gate_depth,
            self.gate_volume,
            self.multiqubit_gate_depth,
            self.program_duration,
            self.program_fidelity,
            self.topological_swaps,
            self.qpu_runtime_estimation,
        )
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[cfg_attr(
    feature = "python",
    pyclass(module = "qcs_sdk.compiler.quilc", frozen, get_all)
)]
pub(crate) struct CompilationResult {
    program: String,
    native_quil_metadata: Option<NativeQuilMetadata>,
}

py_function_sync_async! {
    #[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk.compiler.quilc"))]
    #[pyfunction]
    async fn get_version_info(client: PyQuilcClient) -> PyResult<String> {
        client.inner.as_client().get_version_info().map_err(Into::into)
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PauliTerm {
    #[new]
    fn __new__(indices: Vec<u64>, symbols: Vec<String>) -> Self {
        Self { indices, symbols }
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl ConjugateByCliffordRequest {
    #[new]
    fn __new__(pauli: PauliTerm, clifford: String) -> Self {
        Self { pauli, clifford }
    }
}

py_function_sync_async! {
    #[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk.compiler.quilc"))]
    #[pyfunction]
    async fn conjugate_pauli_by_clifford(
        request: ConjugateByCliffordRequest,
        client: PyQuilcClient,
    ) -> PyResult<ConjugatePauliByCliffordResponse> {
        client.inner.as_client()
            .conjugate_pauli_by_clifford(request)
            .map_err(Into::into)
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl RandomizedBenchmarkingRequest {
    #[new]
    fn __new__(
        depth: u64,
        qubits: u64,
        gateset: Vec<String>,
        seed: Option<u64>,
        interleaver: Option<String>,
    ) -> Self {
        Self {
            depth,
            qubits,
            gateset,
            seed,
            interleaver,
        }
    }
}

py_function_sync_async! {
    #[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk.compiler.quilc"))]
    #[pyfunction]
    async fn generate_randomized_benchmarking_sequence(
        request: RandomizedBenchmarkingRequest,
        client: PyQuilcClient,
    ) -> PyResult<GenerateRandomizedBenchmarkingSequenceResponse> {
        client.inner
            .as_client()
            .generate_randomized_benchmarking_sequence(request)
            .map_err(Into::into)
    }
}
