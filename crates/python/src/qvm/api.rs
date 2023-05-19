use std::collections::HashMap;

use super::RustQvmError;
use crate::{
    py_sync::py_function_sync_async, qpu::client::PyQcsClient, register_data::PyRegisterData,
};

use pyo3::{
    pymethods,
    types::{PyBool, PyBytes, PyFloat, PyInt, PyString},
    Py, Python,
};
use qcs::{
    qvm::api::{
        AddressRequest, ExpectationRequest, ExpectationResponse, MultishotMeasureRequest,
        MultishotRequest, MultishotResponse, WavefunctionRequest, WavefunctionResponse,
    },
    RegisterData,
};
use rigetti_pyo3::{
    create_init_submodule, impl_repr, py_wrap_data_struct, py_wrap_union_enum,
    pyo3::{pyfunction, PyResult},
    PyTryFrom, PyWrapper, ToPythonError,
};

create_init_submodule! {
    classes: [
        PyAddressRequest,
        PyMultishotRequest,
        PyMultishotResponse,
        PyMultishotMeasureRequest,
        PyExpectationRequest,
        PyExpectationResponse,
        PyWavefunctionRequest,
        PyWavefunctionResponse
    ],
    funcs: [
        py_get_version_info,
        py_get_version_info_async,
        py_run,
        py_run_async,
        py_run_and_measure,
        py_run_and_measure_async,
        py_measure_expectation,
        py_measure_expectation_async,
        py_get_wavefunction,
        py_get_wavefunction_async
    ],
}

py_function_sync_async! {
    #[pyfunction(client = "None")]
    async fn get_version_info(client: Option<PyQcsClient>) -> PyResult<String> {
        let client = PyQcsClient::get_or_create_client(client).await?;
        let config = client.get_config();
        qcs::qvm::api::get_version_info(&config)
            .await
            .map_err(RustQvmError::from)
            .map_err(RustQvmError::to_py_err)
    }
}

py_wrap_union_enum! {
    PyAddressRequest(AddressRequest) as "AddressRequest" {
        all: All => Py<PyBool>,
        indices: Indices => Vec<Py<PyInt>>
    }
}
impl_repr!(PyAddressRequest);

py_wrap_data_struct! {
    #[derive(Debug, PartialEq)]
    PyMultishotRequest(MultishotRequest) as "MultishotRequest" {
        quil_instructions: String => Py<PyString>,
        addresses: HashMap<String, AddressRequest> => HashMap<String, PyAddressRequest>,
        trials: u16 => Py<PyInt>,
        measurement_noise: Option<(f64, f64, f64)> => Option<(Py<PyFloat>, Py<PyFloat>, Py<PyFloat>)>,
        gate_noise: Option<(f64, f64, f64)> => Option<(Py<PyFloat>, Py<PyFloat>, Py<PyFloat>)>,
        rng_seed: Option<i64> => Option<Py<PyInt>>
    }
}
impl_repr!(PyMultishotRequest);

#[pymethods]
impl PyMultishotRequest {
    #[new]
    pub fn new(
        py: Python<'_>,
        program: &str,
        shots: u16,
        addresses: HashMap<String, PyAddressRequest>,
        measurement_noise: Option<(f64, f64, f64)>,
        gate_noise: Option<(f64, f64, f64)>,
        rng_seed: Option<i64>,
    ) -> PyResult<Self> {
        Ok(Self(MultishotRequest::new(
            program,
            shots,
            HashMap::<String, AddressRequest>::py_try_from(py, &addresses)?,
            measurement_noise,
            gate_noise,
            rng_seed,
        )))
    }
}

py_wrap_data_struct! {
    #[derive(Debug, PartialEq)]
    PyMultishotResponse(MultishotResponse) as "MultishotResponse" {
        registers: HashMap<String, RegisterData> => HashMap<String, PyRegisterData>
    }
}

py_function_sync_async! {
    #[pyfunction(client = "None")]
    async fn run(
        request: PyMultishotRequest,
        client: Option<PyQcsClient>,
    ) -> PyResult<PyMultishotResponse> {
        let client = PyQcsClient::get_or_create_client(client).await?;
        let config = client.get_config();
        qcs::qvm::api::run(request.as_inner(), &config)
            .await
            .map_err(RustQvmError::from)
            .map_err(RustQvmError::to_py_err)
            .map(|response| PyMultishotResponse(response))
    }
}

py_wrap_data_struct! {
    PyMultishotMeasureRequest(MultishotMeasureRequest) as "MultishotMeasureRequest" {
        quil_instructions: String => Py<PyString>,
        trials: u16 => Py<PyInt>,
        qubits: Vec<u64> => Vec<Py<PyInt>>,
        measurement_noise: Option<(f64, f64, f64)> => Option<(Py<PyFloat>, Py<PyFloat>, Py<PyFloat>)>,
        gate_noise: Option<(f64, f64, f64)> => Option<(Py<PyFloat>, Py<PyFloat>, Py<PyFloat>)>,
        rng_seed: Option<i64> => Option<Py<PyInt>>
    }
}
impl_repr!(PyMultishotMeasureRequest);

#[pymethods]
impl PyMultishotMeasureRequest {
    #[new]
    pub fn new(
        program: &str,
        shots: u16,
        qubits: Vec<u64>,
        measurement_noise: Option<(f64, f64, f64)>,
        gate_noise: Option<(f64, f64, f64)>,
        rng_seed: Option<i64>,
    ) -> PyResult<Self> {
        Ok(Self(MultishotMeasureRequest::new(
            program,
            shots,
            qubits,
            measurement_noise,
            gate_noise,
            rng_seed,
        )))
    }
}

py_function_sync_async! {
    #[pyfunction(client = "None")]
    async fn run_and_measure(request: PyMultishotMeasureRequest, client: Option<PyQcsClient>) -> PyResult<Vec<Vec<i64>>> {
        let client = PyQcsClient::get_or_create_client(client).await?;
        let config = client.get_config();
        qcs::qvm::api::run_and_measure(request.as_inner(), &config)
            .await
            .map_err(RustQvmError::from)
            .map_err(RustQvmError::to_py_err)
    }
}

py_wrap_data_struct! {
    PyExpectationRequest(ExpectationRequest) as "ExpectationRequest" {
        state_preparation: String => Py<PyString>,
        operators: Vec<String> => Vec<Py<PyString>>,
        rng_seed: Option<i64> => Option<Py<PyInt>>
    }
}
impl_repr!(PyExpectationRequest);

#[pymethods]
impl PyExpectationRequest {
    #[new]
    pub fn new(state_preparation: &str, operators: Vec<String>, rng_seed: Option<i64>) -> Self {
        Self(ExpectationRequest::new(
            state_preparation,
            operators,
            rng_seed,
        ))
    }
}

py_wrap_data_struct! {
    PyExpectationResponse(ExpectationResponse) as "ExpectationResponse" {
        expectations: Vec<f64> => Vec<Py<PyFloat>>
    }
}
impl_repr!(PyExpectationResponse);

py_function_sync_async! {
    #[pyfunction(client = "None")]
    async fn measure_expectation(request: PyExpectationRequest, client: Option<PyQcsClient>) -> PyResult<PyExpectationResponse> {
        let config = PyQcsClient::get_or_create_client(client).await?.get_config();
        qcs::qvm::api::measure_expectation(request.as_inner(), &config)
            .await
            .map_err(RustQvmError::from)
            .map_err(RustQvmError::to_py_err)
            .map(|response| PyExpectationResponse(response))
    }
}

py_wrap_data_struct! {
    PyWavefunctionRequest(WavefunctionRequest) as "WavefunctionRequest" {
        compiled_quil: String => Py<PyString>,
        measurement_noise: Option<(f64, f64, f64)> => Option<(Py<PyFloat>, Py<PyFloat>, Py<PyFloat>)>,
        gate_noise: Option<(f64, f64, f64)> => Option<(Py<PyFloat>, Py<PyFloat>, Py<PyFloat>)>,
        rng_seed: Option<i64> => Option<Py<PyInt>>
    }
}
impl_repr!(PyWavefunctionRequest);

#[pymethods]
impl PyWavefunctionRequest {
    #[new]
    fn new(
        compiled_quil: &str,
        measurement_noise: Option<(f64, f64, f64)>,
        gate_noise: Option<(f64, f64, f64)>,
        rng_seed: Option<i64>,
    ) -> Self {
        Self(WavefunctionRequest::new(
            compiled_quil,
            measurement_noise,
            gate_noise,
            rng_seed,
        ))
    }
}

py_wrap_data_struct! {
    PyWavefunctionResponse(WavefunctionResponse) as "WavefunctionResponse" {
        wavefunction: Vec<u8> => Py<PyBytes>
    }
}
impl_repr!(PyWavefunctionResponse);

py_function_sync_async! {
    #[pyfunction(client = "None")]
    async fn get_wavefunction(request: PyWavefunctionRequest, client: Option<PyQcsClient>) -> PyResult<PyWavefunctionResponse> {
        let config = PyQcsClient::get_or_create_client(client).await?.get_config();
        qcs::qvm::api::get_wavefunction(request.as_inner(), &config)
            .await
            .map_err(RustQvmError::from)
            .map_err(RustQvmError::to_py_err)
            .map(|response| PyWavefunctionResponse(response))
    }
}
