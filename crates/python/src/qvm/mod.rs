use qcs::{
    qvm::{self, QvmOptions, QvmResultData},
    RegisterData,
};
use rigetti_pyo3::{
    create_init_submodule, impl_as_mut_for_wrapper, impl_repr, py_wrap_error, py_wrap_type,
    pyo3::{exceptions::PyRuntimeError, prelude::*, Python},
    wrap_error, PyTryFrom, PyWrapper, PyWrapperMut, ToPython, ToPythonError,
};
use std::num::NonZeroU16;
use std::{collections::HashMap, time::Duration};

use crate::{py_sync::py_function_sync_async, register_data::PyRegisterData};

mod api;

use api::PyAddressRequest;

wrap_error!(RustQvmError(qcs::qvm::Error));
py_wrap_error!(api, RustQvmError, QVMError, PyRuntimeError);

py_wrap_type! {
    PyQvmResultData(QvmResultData) as "QVMResultData"
}

create_init_submodule! {
    classes: [PyQvmResultData, PyQvmOptions, PyQvmHttpClient],
    errors: [QVMError],
    funcs: [py_run, py_run_async],
    submodules: [
        "api": api::init_submodule
    ],
}

#[pyo3::pyclass]
#[pyo3(name = "QVMHTTPClient")]
#[derive(Debug, Clone)]
pub struct PyQvmHttpClient(pub qvm::http::HttpClient);

#[pymethods]
impl PyQvmHttpClient {
    #[new]
    pub fn new(address: String) -> Self {
        Self(qvm::http::HttpClient::new(address))
    }
}

#[derive(Debug, pyo3::FromPyObject)]
pub enum QvmClient {
    Http(PyQvmHttpClient),
}

#[pymethods]
impl PyQvmResultData {
    #[staticmethod]
    fn from_memory_map(py: Python<'_>, memory: HashMap<String, PyRegisterData>) -> PyResult<Self> {
        Ok(Self(QvmResultData::from_memory_map(HashMap::<
            String,
            RegisterData,
        >::py_try_from(
            py, &memory
        )?)))
    }

    #[getter]
    fn memory(&self, py: Python<'_>) -> PyResult<HashMap<String, PyRegisterData>> {
        self.as_inner().memory().to_python(py)
    }
}

py_wrap_type! {
    #[derive(Default)]
    PyQvmOptions(QvmOptions) as "QVMOptions"
}
impl_as_mut_for_wrapper!(PyQvmOptions);
impl_repr!(PyQvmOptions);

#[pymethods]
impl PyQvmOptions {
    #[new]
    #[args("/", timeout_seconds = "30.0")]
    pub fn new(timeout_seconds: Option<f64>) -> Self {
        Self(QvmOptions {
            timeout: timeout_seconds.map(Duration::from_secs_f64),
        })
    }

    #[getter]
    pub fn timeout(&self) -> Option<f32> {
        self.as_inner()
            .timeout
            .map(|duration| duration.as_secs_f32())
    }

    #[setter]
    pub fn set_timeout(&mut self, timeout_seconds: Option<f64>) {
        self.as_inner_mut().timeout = timeout_seconds.map(Duration::from_secs_f64);
    }

    #[staticmethod]
    #[pyo3(name = "default")]
    pub fn py_default() -> Self {
        <Self as Default>::default()
    }
}

py_function_sync_async! {
    #[pyfunction]
    async fn run(
        quil: String,
        #[pyo3(from_py_with = "crate::from_py::non_zero_u16")]
        shots: NonZeroU16,
        addresses: HashMap<String, PyAddressRequest>,
        params: HashMap<String, Vec<f64>>,
        client: QvmClient,
        measurement_noise: Option<(f64, f64, f64)>,
        gate_noise: Option<(f64, f64, f64)>,
        rng_seed: Option<i64>,
        options: Option<PyQvmOptions>,
    ) -> PyResult<PyQvmResultData> {
        let QvmClient::Http(client) = client;
        let params = params.into_iter().map(|(key, value)| (key.into_boxed_str(), value)).collect();
        let addresses = addresses.into_iter().map(|(address, request)| (address, request.as_inner().clone())).collect();
        let options = options.unwrap_or_default();
        Ok(
            PyQvmResultData(
                qcs::qvm::run(
                    &quil,
                    shots,
                    addresses,
                    &params,
                    measurement_noise,
                    gate_noise,
                    rng_seed,
                    &client.0,
                    options.as_inner()
            )
            .await
            .map_err(RustQvmError::from)
            .map_err(RustQvmError::to_py_err)?))
    }
}
