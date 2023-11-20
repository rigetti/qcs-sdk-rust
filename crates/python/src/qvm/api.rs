use std::{collections::HashMap, num::NonZeroU16};

use super::{PyQvmOptions, RustQvmError};
use crate::{py_sync::py_function_sync_async, register_data::PyRegisterData};

use pyo3::{
    pymethods,
    types::{PyFloat, PyInt, PyString},
    Py, Python,
};
use qcs::{
    qvm::{
        http::{
            AddressRequest, ExpectationRequest, MultishotMeasureRequest, MultishotRequest,
            MultishotResponse, WavefunctionRequest,
        },
        Client,
    },
    RegisterData,
};
use rigetti_pyo3::{
    create_init_submodule, impl_repr, py_wrap_data_struct, py_wrap_type,
    pyo3::{pyfunction, PyResult},
    PyTryFrom, PyWrapper, PyWrapperMut, ToPythonError,
};

create_init_submodule! {
    classes: [
        PyAddressRequest,
        PyMultishotRequest,
        PyMultishotResponse,
        PyMultishotMeasureRequest,
        PyExpectationRequest,
        PyWavefunctionRequest
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
    #[pyfunction]
    #[pyo3(signature = (client, options = None))]
    async fn get_version_info(client: super::PyQvmClient, options: Option<PyQvmOptions>) -> PyResult<String> {
        client.get_version_info(options.unwrap_or_default().as_inner())
            .await
            .map_err(RustQvmError::from)
            .map_err(RustQvmError::to_py_err)
    }
}

py_wrap_type! {
    PyAddressRequest(AddressRequest) as "AddressRequest"
}
impl_repr!(PyAddressRequest);

#[pymethods]
impl PyAddressRequest {
    #[staticmethod]
    pub fn include_all() -> Self {
        Self(AddressRequest::IncludeAll)
    }

    #[staticmethod]
    pub fn exclude_all() -> Self {
        Self(AddressRequest::ExcludeAll)
    }

    #[staticmethod]
    pub fn from_indices(indices: Vec<usize>) -> Self {
        Self(AddressRequest::Indices(indices))
    }
}

py_wrap_data_struct! {
    #[derive(Debug, PartialEq)]
    PyMultishotRequest(MultishotRequest) as "MultishotRequest" {
        compiled_quil: String => Py<PyString>,
        addresses: HashMap<String, AddressRequest> => HashMap<String, PyAddressRequest>,
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
        program: String,
        #[pyo3(from_py_with = "crate::from_py::non_zero_u16")] shots: NonZeroU16,
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

    #[getter]
    pub fn trials(&self) -> u16 {
        self.as_inner().trials.get()
    }

    #[setter]
    pub fn set_trials(&mut self, trials: u16) -> PyResult<()> {
        // `NonZeroU16` doesn't implement `PyClass`, so `pyo3` doesn't allow it to be used
        // as a method argument, even when combined with a `from_py_with` attribute.
        self.as_inner_mut().trials = crate::from_py::try_from_u16_to_non_zero_u16(trials)?;
        Ok(())
    }
}

py_wrap_data_struct! {
    #[derive(Debug, PartialEq)]
    PyMultishotResponse(MultishotResponse) as "MultishotResponse" {
        registers: HashMap<String, RegisterData> => HashMap<String, PyRegisterData>
    }
}
impl_repr!(PyMultishotResponse);

py_function_sync_async! {
    #[pyfunction]
    #[pyo3(signature = (request, client, options = None))]
    async fn run(
        request: PyMultishotRequest,
        client: super::PyQvmClient,
        options: Option<PyQvmOptions>,
    ) -> PyResult<PyMultishotResponse> {
        client.run(request.as_inner(), options.unwrap_or_default().as_inner())
            .await
            .map_err(RustQvmError::from)
            .map_err(RustQvmError::to_py_err)
            .map(PyMultishotResponse)
    }
}

py_wrap_data_struct! {
    PyMultishotMeasureRequest(MultishotMeasureRequest) as "MultishotMeasureRequest" {
        compiled_quil: String => Py<PyString>,
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
        program: String,
        #[pyo3(from_py_with = "crate::from_py::non_zero_u16")] shots: NonZeroU16,
        qubits: Vec<u64>,
        measurement_noise: Option<(f64, f64, f64)>,
        gate_noise: Option<(f64, f64, f64)>,
        rng_seed: Option<i64>,
    ) -> PyResult<Self> {
        Ok(Self(MultishotMeasureRequest::new(
            program,
            shots,
            &qubits,
            measurement_noise,
            gate_noise,
            rng_seed,
        )))
    }

    #[getter]
    pub fn trials(&self) -> u16 {
        self.as_inner().trials.get()
    }

    #[setter]
    pub fn set_trials(&mut self, trials: u16) -> PyResult<()> {
        // `NonZeroU16` doesn't implement `PyClass`, so `pyo3` doesn't allow it to be used
        // as a method argument, even when combined with a `from_py_with` attribute.
        self.as_inner_mut().trials = crate::from_py::try_from_u16_to_non_zero_u16(trials)?;
        Ok(())
    }
}

py_function_sync_async! {
    #[pyfunction]
    #[pyo3(signature = (request, client, options = None))]
    async fn run_and_measure(request: PyMultishotMeasureRequest, client: super::PyQvmClient, options: Option<PyQvmOptions>) -> PyResult<Vec<Vec<i64>>> {
        client.run_and_measure(request.as_inner(), options.unwrap_or_default().as_inner())
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
    pub fn new(state_preparation: String, operators: Vec<String>, rng_seed: Option<i64>) -> Self {
        Self(ExpectationRequest::new(
            state_preparation,
            &operators,
            rng_seed,
        ))
    }
}

py_function_sync_async! {
    #[pyfunction]
    #[pyo3(signature = (request, client, options = None))]
    async fn measure_expectation(request: PyExpectationRequest, client: super::PyQvmClient, options: Option<PyQvmOptions>) -> PyResult<Vec<f64>> {
        client.measure_expectation(request.as_inner(), options.unwrap_or_default().as_inner())
            .await
            .map_err(RustQvmError::from)
            .map_err(RustQvmError::to_py_err)
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
        compiled_quil: String,
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

py_function_sync_async! {
    #[pyfunction]
    #[pyo3(signature = (request, client, options = None))]
    async fn get_wavefunction(request: PyWavefunctionRequest, client: super::PyQvmClient, options: Option<PyQvmOptions>) -> PyResult<Vec<u8>> {
        client.get_wavefunction(request.as_inner(), options.unwrap_or_default().as_inner())
            .await
            .map_err(RustQvmError::from)
            .map_err(RustQvmError::to_py_err)
    }
}
