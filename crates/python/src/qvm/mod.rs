use pyo3::types::PyList;
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

create_init_submodule! {
    classes: [PyQvmResultData, PyQvmOptions, RawQvmReadoutData, PyQvmClient],
    errors: [QVMError],
    funcs: [py_run, py_run_async],
    submodules: [
        "api": api::init_submodule
    ],
}

#[derive(Clone)]
pub enum QvmClient {
    Http(qvm::http::HttpClient),
}

#[pyclass(name = "QVMClient")]
#[derive(Clone)]
pub struct PyQvmClient {
    pub inner: QvmClient,
}

#[pymethods]
impl PyQvmClient {
    #[new]
    fn new() -> PyResult<Self> {
        Err(PyRuntimeError::new_err("QVMClient cannot be instantiated directly. See the static methods: QVMClient.new_http()."))
    }

    #[staticmethod]
    fn new_http(endpoint: &str) -> PyResult<Self> {
        let http_client = qvm::http::HttpClient::new(endpoint.to_string());
        Ok(Self {
            inner: QvmClient::Http(http_client),
        })
    }

    #[getter]
    fn qvm_url(&self) -> PyResult<String> {
        let QvmClient::Http(client) = &self.inner;
        Ok(client.qvm_url.to_string())
    }
}

wrap_error!(RustQvmError(qcs::qvm::Error));
py_wrap_error!(api, RustQvmError, QVMError, PyRuntimeError);

py_wrap_type! {
    PyQvmResultData(QvmResultData) as "QVMResultData"
}

#[pymethods]
impl PyQvmResultData {
    #[new]
    fn new(memory: HashMap<String, PyRegisterData>) -> Self {
        let memory = memory
            .into_iter()
            .map(|(key, value)| (key, value.into_inner()))
            .collect();
        Self::from(QvmResultData::from_memory_map(memory))
    }

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

    pub(crate) fn to_raw_readout_data(&self, py: Python<'_>) -> RawQvmReadoutData {
        RawQvmReadoutData {
            memory: self
                .as_inner()
                .memory()
                .iter()
                .map(|(register, matrix)| {
                    (
                        register.to_string(),
                        match matrix {
                            RegisterData::I8(matrix) => PyList::new(py, matrix).into_py(py),
                            RegisterData::F64(matrix) => PyList::new(py, matrix).into_py(py),
                            RegisterData::I16(matrix) => PyList::new(py, matrix).into_py(py),
                            RegisterData::Complex32(matrix) => PyList::new(py, matrix).into_py(py),
                        },
                    )
                })
                .collect::<HashMap<String, Py<PyList>>>(),
        }
    }
}

#[pyclass(name = "RawQVMReadoutData")]
#[derive(Debug)]
pub(crate) struct RawQvmReadoutData {
    #[pyo3(get)]
    memory: HashMap<String, Py<PyList>>,
}

impl RawQvmReadoutData {
    fn __repr__(&self) -> String {
        format!("{self:?}")
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
    #[pyo3(signature = (timeout_seconds = None))]
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
        client: PyQvmClient,
        measurement_noise: Option<(f64, f64, f64)>,
        gate_noise: Option<(f64, f64, f64)>,
        rng_seed: Option<i64>,
        options: Option<PyQvmOptions>,
    ) -> PyResult<PyQvmResultData> {
        let QvmClient::Http(client) = client.inner;
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
                    &client,
                    options.as_inner()
            )
            .await
            .map_err(RustQvmError::from)
            .map_err(RustQvmError::to_py_err)?))
    }
}
