use qcs::compiler::quilc::{
    CompilerOpts, ConjugateByCliffordRequest, ConjugatePauliByCliffordResponse,
    GenerateRandomizedBenchmarkingSequenceResponse, NativeQuilMetadata, PauliTerm,
    RandomizedBenchmarkingRequest, TargetDevice, DEFAULT_COMPILER_TIMEOUT,
};
use qcs_api_client_openapi::models::InstructionSetArchitecture;
use rigetti_pyo3::{
    create_init_submodule, impl_repr, py_wrap_data_struct, py_wrap_error, py_wrap_struct,
    py_wrap_type,
    pyo3::{
        exceptions::{PyRuntimeError, PyValueError},
        pyclass, pyfunction, pymethods,
        types::{PyBytes, PyFloat, PyInt, PyString},
        Py, PyResult, Python,
    },
    wrap_error, PyWrapper, ToPythonError,
};

use crate::py_sync::py_function_sync_async;
use crate::qpu::client::PyQcsClient;
use crate::qpu::isa::PyInstructionSetArchitecture;

create_init_submodule! {
    classes: [
        PyCompilerOpts,
        PyCompilationResult,
        PyNativeQuilMetadata,
        PyTargetDevice,
        PyPauliTerm,
        PyConjugateByCliffordRequest,
        PyConjugatePauliByCliffordResponse,
        PyRandomizedBenchmarkingRequest,
        PyGenerateRandomizedBenchmarkingSequenceResponse
    ],
    consts: [DEFAULT_COMPILER_TIMEOUT],
    errors: [QuilcError],
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

py_wrap_type! {
    #[derive(Default)]
    PyCompilerOpts(CompilerOpts) as "CompilerOpts";
}

#[pymethods]
impl PyCompilerOpts {
    #[new]
    #[args("/", timeout = "DEFAULT_COMPILER_TIMEOUT", protoquil = "None")]
    pub fn new(timeout: Option<f64>, protoquil: Option<bool>) -> Self {
        let opts = CompilerOpts::new()
            .with_timeout(timeout)
            .with_protoquil(protoquil);
        Self(opts)
    }

    #[staticmethod]
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Self {
        <Self as Default>::default()
    }
}

wrap_error!(RustQuilcError(qcs::compiler::quilc::Error));
py_wrap_error!(quilc, RustQuilcError, QuilcError, PyRuntimeError);

py_wrap_struct! {
    PyTargetDevice(TargetDevice) as "TargetDevice" {}
}

#[pymethods]
impl PyTargetDevice {
    #[staticmethod]
    pub fn from_isa(isa: PyInstructionSetArchitecture) -> PyResult<Self> {
        let isa: InstructionSetArchitecture = isa.into();
        let target: TargetDevice = isa
            .try_into()
            .map_err(RustQuilcError::from)
            .map_err(RustQuilcError::to_py_err)?;

        Ok(Self(target))
    }

    #[staticmethod]
    pub fn from_json(value: String) -> PyResult<Self> {
        let target: TargetDevice = serde_json::from_str(&value)
            .map_err(|err| err.to_string())
            .map_err(PyValueError::new_err)?;

        Ok(Self(target))
    }
}

py_function_sync_async! {
    #[pyfunction(client = "None", options = "None")]
    async fn compile_program(
        quil: String,
        target: PyTargetDevice,
        client: Option<PyQcsClient>,
        options: Option<PyCompilerOpts>,
    ) -> PyResult<PyCompilationResult> {
        let client = PyQcsClient::get_or_create_client(client).await?;
        let options = options.unwrap_or_default();
        qcs::compiler::quilc::compile_program(&quil, target.into(), &client, options.into())
            .map_err(RustQuilcError::from)
            .map_err(RustQuilcError::to_py_err)
            .map(|result| PyCompilationResult {
                program: result.program.to_string(true),
                native_quil_metadata: result.native_quil_metadata.map(PyNativeQuilMetadata)
            })

    }
}

py_wrap_data_struct! {
    #[derive(Debug, PartialEq, PartialOrd)]
    PyNativeQuilMetadata(NativeQuilMetadata) as "NativeQuilMetadata" {
        final_rewiring: Vec<u64> => Vec<Py<PyInt>>,
        gate_depth: Option<u64> => Option<Py<PyInt>>,
        gate_volume: Option<u64> => Option<Py<PyInt>>,
        multiqubit_gate_depth: Option<u64> => Option<Py<PyInt>>,
        program_duration: Option<f64> => Option<Py<PyFloat>>,
        program_fidelity: Option<f64> => Option<Py<PyFloat>>,
        topological_swaps: Option<u64> => Option<Py<PyInt>>,
        qpu_runtime_estimation: Option<f64> => Option<Py<PyFloat>>

    }
}
impl_repr!(PyNativeQuilMetadata);

#[pymethods]
impl PyNativeQuilMetadata {
    #[new]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        final_rewiring: Option<Vec<u64>>,
        gate_depth: Option<u64>,
        gate_volume: Option<u64>,
        multiqubit_gate_depth: Option<u64>,
        program_duration: Option<f64>,
        program_fidelity: Option<f64>,
        topological_swaps: Option<u64>,
        qpu_runtime_estimation: Option<f64>,
    ) -> Self {
        Self(NativeQuilMetadata {
            final_rewiring: final_rewiring.unwrap_or_default(),
            gate_depth,
            gate_volume,
            multiqubit_gate_depth,
            program_duration,
            program_fidelity,
            topological_swaps,
            qpu_runtime_estimation,
        })
    }
    pub fn __getstate__<'a>(&self, py: Python<'a>) -> PyResult<&'a PyBytes> {
        Ok(PyBytes::new(
            py,
            &serde_json::to_vec(self.as_inner())
                .map_err(|e| PyRuntimeError::new_err(format!("failed to serialize: {e}")))?,
        ))
    }

    pub fn __setstate__(&mut self, state: &PyBytes) -> PyResult<()> {
        let metadata: NativeQuilMetadata = serde_json::from_slice(state.as_bytes())
            .map_err(|e| PyRuntimeError::new_err(format!("failed to deserialize: {e}")))?;
        *self = Self(metadata);
        Ok(())
    }
}

#[pyclass(name = "CompilationResult")]
pub struct PyCompilationResult {
    #[pyo3(get)]
    program: String,
    #[pyo3(get)]
    native_quil_metadata: Option<PyNativeQuilMetadata>,
}

py_function_sync_async! {
    #[pyfunction(client = "None")]
    async fn get_version_info(
        client: Option<PyQcsClient>,
    ) -> PyResult<String> {
        let client = PyQcsClient::get_or_create_client(client).await?;
        qcs::compiler::quilc::get_version_info(&client)
            .map_err(RustQuilcError::from)
            .map_err(RustQuilcError::to_py_err)
    }
}

py_wrap_data_struct! {
    PyPauliTerm(PauliTerm) as "PauliTerm" {
        indices: Vec<u64> => Vec<Py<PyInt>>,
        symbols: Vec<String> => Vec<Py<PyString>>
    }
}

#[pymethods]
impl PyPauliTerm {
    #[new]
    fn __new__(indices: Vec<u64>, symbols: Vec<String>) -> PyResult<Self> {
        Ok(Self(PauliTerm { indices, symbols }))
    }
}

py_wrap_data_struct! {
    PyConjugateByCliffordRequest(ConjugateByCliffordRequest) as "ConjugateByCliffordRequest" {
        pauli: PauliTerm => PyPauliTerm,
        clifford: String => Py<PyString>
    }
}

#[pymethods]
impl PyConjugateByCliffordRequest {
    #[new]
    fn __new__(pauli: PyPauliTerm, clifford: String) -> PyResult<Self> {
        Ok(Self(ConjugateByCliffordRequest {
            pauli: pauli.into(),
            clifford,
        }))
    }
}

py_wrap_data_struct! {
    PyConjugatePauliByCliffordResponse(ConjugatePauliByCliffordResponse) as "ConjugatePauliByCliffordResponse" {
        phase: i64 => Py<PyInt>,
        pauli: String => Py<PyString>
    }
}

py_function_sync_async! {
    #[pyfunction(client = "None")]
    async fn conjugate_pauli_by_clifford(
        request: PyConjugateByCliffordRequest,
        client: Option<PyQcsClient>,
    ) -> PyResult<PyConjugatePauliByCliffordResponse> {
        let client = PyQcsClient::get_or_create_client(client).await?;
        qcs::compiler::quilc::conjugate_pauli_by_clifford(&client, request.into())
            .map(PyConjugatePauliByCliffordResponse::from)
            .map_err(RustQuilcError::from)
            .map_err(RustQuilcError::to_py_err)
    }
}

py_wrap_data_struct! {
    PyRandomizedBenchmarkingRequest(RandomizedBenchmarkingRequest) as "RandomizedBenchmarkingRequest" {
        depth: u64 => Py<PyInt>,
        qubits: u64 => Py<PyInt>,
        gateset: Vec<String> => Vec<Py<PyString>>,
        seed: Option<u64> => Option<Py<PyInt>>,
        interleaver: Option<String> => Option<Py<PyString>>
    }
}

#[pymethods]
impl PyRandomizedBenchmarkingRequest {
    #[new]
    fn __new__(
        depth: u64,
        qubits: u64,
        gateset: Vec<String>,
        seed: Option<u64>,
        interleaver: Option<String>,
    ) -> PyResult<Self> {
        Ok(Self(RandomizedBenchmarkingRequest {
            depth,
            qubits,
            gateset,
            seed,
            interleaver,
        }))
    }
}

py_wrap_data_struct! {
    PyGenerateRandomizedBenchmarkingSequenceResponse(GenerateRandomizedBenchmarkingSequenceResponse) as "GenerateRandomizedBenchmarkingSequenceResponse" {
        sequence: Vec<Vec<i64>> => Vec<Vec<Py<PyInt>>>
    }
}

py_function_sync_async! {
    #[pyfunction(client = "None")]
    async fn generate_randomized_benchmarking_sequence(
        request: PyRandomizedBenchmarkingRequest,
        client: Option<PyQcsClient>,
    ) -> PyResult<PyGenerateRandomizedBenchmarkingSequenceResponse> {
        let client = PyQcsClient::get_or_create_client(client).await?;
        qcs::compiler::quilc::generate_randomized_benchmarking_sequence(&client, request.into())
            .map(PyGenerateRandomizedBenchmarkingSequenceResponse::from)
            .map_err(RustQuilcError::from)
            .map_err(RustQuilcError::to_py_err)
    }
}
