use std::collections::HashMap;
use std::time::Duration;

use pyo3::{prelude::*, types::PyList, Py};

#[cfg(feature = "stubs")]
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyfunction, gen_stub_pymethods};

use crate::{
    python::{errors, impl_repr, py_function_sync_async, NonZeroU16},
    qvm::{self, http, Error, QvmOptions, QvmResultData},
    register_data::RegisterData,
};

#[pymodule]
#[pyo3(name = "qvm", module = "qcs_sdk", submodule)]
pub(crate) fn init_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();

    m.add("QVMError", py.get_type::<errors::QvmError>())?;

    m.add_class::<QvmResultData>()?;
    m.add_class::<QvmOptions>()?;
    m.add_class::<RawQvmReadoutData>()?;
    m.add_class::<PyQvmClient>()?;

    m.add_function(wrap_pyfunction!(py_run, m)?)?;
    m.add_function(wrap_pyfunction!(py_run_async, m)?)?;

    let submodule = PyModule::new(py, "api")?;
    // m.add_wrapped(wrap_pymodule!(api::init_module))?;
    api::init_module(&submodule)?;
    m.add_submodule(&submodule)?;

    Ok(())
}

impl_repr!(QvmOptions);
impl_repr!(RawQvmReadoutData);

#[derive(Clone)]
pub enum QvmClient {
    Http(http::HttpClient),
    #[cfg(feature = "libquil")]
    Libquil(qvm::libquil::Client),
}

#[derive(Clone)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(name = "QVMClient", module = "qcs_sdk.qvm")]
pub struct PyQvmClient {
    inner: QvmClient,
}

#[derive(Debug)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyo3::pyclass(name = "RawQVMReadoutData", module = "qcs_sdk.qvm", frozen, get_all)]
pub struct RawQvmReadoutData {
    memory: HashMap<String, Py<PyList>>,
}

impl PyQvmClient {
    pub fn as_client(&self) -> &(dyn qvm::Client + Send + Sync) {
        match &self.inner {
            QvmClient::Http(client) => client,
            #[cfg(feature = "libquil")]
            QvmClient::Libquil(client) => client,
        }
    }
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyQvmClient {
    #[new]
    fn new() -> PyResult<Self> {
        Err(errors::QvmError::new_err(
                "QVMClient cannot be instantiated directly. See the static methods: QVMClient.new_http() and QVMClient.new_libquil()."
                ))
    }

    #[staticmethod]
    fn new_http(endpoint: &str) -> Self {
        let http_client = http::HttpClient::new(endpoint.to_string());
        Self {
            inner: QvmClient::Http(http_client),
        }
    }

    #[cfg(feature = "libquil")]
    #[staticmethod]
    fn new_libquil() -> Self {
        Self {
            inner: QvmClient::Libquil(qvm::libquil::Client {}),
        }
    }

    #[cfg(not(feature = "libquil"))]
    #[staticmethod]
    fn new_libquil() -> PyResult<Self> {
        Err(errors::QvmError::new_err(
            "Cannot create a libquil QVM client as feature is not enabled.",
        ))
    }

    #[getter]
    fn qvm_url(&self) -> String {
        match &self.inner {
            QvmClient::Http(client) => client.qvm_url.to_string(),
            #[cfg(feature = "libquil")]
            QvmClient::Libquil(_) => "".into(),
        }
    }
}

#[async_trait::async_trait]
impl qvm::Client for PyQvmClient {
    /// The QVM version string. Not guaranteed to comply to the semver spec.
    async fn get_version_info(&self, options: &QvmOptions) -> Result<String, Error> {
        self.as_client().get_version_info(options).await
    }

    /// Execute a program on the QVM.
    async fn run(
        &self,
        request: &http::MultishotRequest,
        options: &QvmOptions,
    ) -> Result<http::MultishotResponse, Error> {
        self.as_client().run(request, options).await
    }

    /// Execute a program on the QVM.
    ///
    /// The behavior of this method is different to that of [`Self::run`]
    /// in that [`Self::run_and_measure`] will execute the program a single
    /// time; the resulting wavefunction is then sampled some number of times
    /// (specified in [`http::MultishotMeasureRequest`]).
    ///
    /// This can be useful if the program is expensive to execute and does
    /// not change per "shot".
    async fn run_and_measure(
        &self,
        request: &http::MultishotMeasureRequest,
        options: &QvmOptions,
    ) -> Result<Vec<Vec<i64>>, Error> {
        self.as_client().run_and_measure(request, options).await
    }

    /// Measure the expectation value of a program
    async fn measure_expectation(
        &self,
        request: &http::ExpectationRequest,
        options: &QvmOptions,
    ) -> Result<Vec<f64>, Error> {
        self.as_client().measure_expectation(request, options).await
    }

    /// Get the wavefunction produced by a program
    async fn get_wavefunction(
        &self,
        request: &http::WavefunctionRequest,
        options: &QvmOptions,
    ) -> Result<Vec<u8>, Error> {
        self.as_client().get_wavefunction(request, options).await
    }
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl QvmOptions {
    #[new]
    #[pyo3(signature = (timeout_seconds = None))]
    fn __new__(timeout_seconds: Option<f64>) -> Self {
        Self {
            timeout: timeout_seconds.map(Duration::from_secs_f64),
        }
    }

    #[getter]
    pub fn timeout(&self) -> Option<f32> {
        self.timeout.map(|duration| duration.as_secs_f32())
    }

    #[setter]
    pub fn set_timeout(&mut self, timeout_seconds: Option<f64>) {
        self.timeout = timeout_seconds.map(Duration::from_secs_f64);
    }

    #[staticmethod]
    #[pyo3(name = "default")]
    fn py_default() -> Self {
        Self::default()
    }
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl QvmResultData {
    /// Construct a new `QVMResultData` from a memory map.
    #[new]
    fn __new__(memory: HashMap<String, RegisterData>) -> Self {
        QvmResultData::from_memory_map(memory)
    }

    /// Get the raw readout data as a flattened structure.
    pub fn to_raw_readout_data<'py>(&self, py: Python<'py>) -> PyResult<RawQvmReadoutData> {
        let memory = self
            .memory()
            .iter()
            .map(|(register, data)| {
                (match data {
                    RegisterData::I8(matrix) => PyList::new(py, matrix),
                    RegisterData::F64(matrix) => PyList::new(py, matrix),
                    RegisterData::I16(matrix) => PyList::new(py, matrix),
                    RegisterData::Complex32(matrix) => PyList::new(py, matrix),
                })
                .map(|list| (register.clone(), list.unbind()))
            })
            .collect::<PyResult<_>>()?;

        Ok(RawQvmReadoutData { memory })
    }
}

py_function_sync_async! {
    #[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
    #[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk.qvm"))]
    #[pyfunction]
    #[tracing::instrument(skip_all)]
    // TODO #[pyo3_opentelemetry::pypropagate(on_context_extraction_failure="ignore")]
    async fn run(
        quil: String,
        shots: NonZeroU16,
        addresses: HashMap<String, http::AddressRequest>,
        params: HashMap<String, Vec<f64>>,
        client: PyQvmClient,
        measurement_noise: Option<(f64, f64, f64)>,
        gate_noise: Option<(f64, f64, f64)>,
        rng_seed: Option<i64>,
        options: Option<QvmOptions>,
    ) -> PyResult<QvmResultData> {
        let params = params
            .into_iter()
            .map(|(key, value)| (key.into_boxed_str(), value))
            .collect();

        let options = options.unwrap_or_default();

        qvm::run(
            &quil,
            shots.0,
            addresses,
            &params,
            measurement_noise,
            gate_noise,
            rng_seed,
            &client,
            &options,
        )
        .await
        .map_err(Into::into)
    }
}

mod api {
    use pyo3::prelude::*;

    #[cfg(feature = "stubs")]
    use pyo3_stub_gen::derive::gen_stub_pyfunction;

    use crate::{
        python::{errors, py_function_sync_async},
        qvm::{http, python::PyQvmClient, Client, QvmOptions},
    };

    // #[pymodule]
    // #[pyo3(name = "api", module = "qcs_sdk.qvm", submodule)]
    pub(crate) fn init_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
        let py = m.py();

        m.add("QuilcError", py.get_type::<errors::QuilcError>())?;

        m.add_class::<http::AddressRequest>()?;
        m.add_class::<http::MultishotRequest>()?;
        m.add_class::<http::MultishotResponse>()?;
        m.add_class::<http::MultishotMeasureRequest>()?;
        m.add_class::<http::ExpectationRequest>()?;
        m.add_class::<http::WavefunctionRequest>()?;

        m.add_function(wrap_pyfunction!(py_get_version_info, m)?)?;
        m.add_function(wrap_pyfunction!(py_get_version_info_async, m)?)?;
        m.add_function(wrap_pyfunction!(py_run, m)?)?;
        m.add_function(wrap_pyfunction!(py_run_async, m)?)?;
        m.add_function(wrap_pyfunction!(py_run_and_measure, m)?)?;
        m.add_function(wrap_pyfunction!(py_run_and_measure_async, m)?)?;
        m.add_function(wrap_pyfunction!(py_measure_expectation, m)?)?;
        m.add_function(wrap_pyfunction!(py_measure_expectation_async, m)?)?;
        m.add_function(wrap_pyfunction!(py_get_wavefunction, m)?)?;
        m.add_function(wrap_pyfunction!(py_get_wavefunction_async, m)?)?;

        Ok(())
    }

    py_function_sync_async! {
        #[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
        #[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk.qvm.api"))]
        #[pyfunction]
        #[pyo3(signature = (client, options = None))]
        async fn get_version_info(client: PyQvmClient, options: Option<QvmOptions>) -> PyResult<String> {
            client
                .get_version_info(&options.unwrap_or_default())
                .await
                .map_err(Into::into)
        }
    }

    py_function_sync_async! {
        #[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
        #[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk.qvm.api"))]
        #[pyfunction]
        #[pyo3(signature = (request, client, options = None))]
        #[tracing::instrument(skip_all)]
        // TODO #[pyo3_opentelemetry::pypropagate(on_context_extraction_failure="ignore")]
        async fn get_wavefunction(
            request: http::WavefunctionRequest,
            client: PyQvmClient,
            options: Option<QvmOptions>
        ) -> PyResult<Vec<u8>> {
            client
                .get_wavefunction(&request, &options.unwrap_or_default())
                .await
                .map_err(Into::into)
        }
    }

    py_function_sync_async! {
        #[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
        #[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk.qvm.api"))]
        #[pyfunction]
        #[pyo3(signature = (request, client, options = None))]
        #[tracing::instrument(skip_all)]
        //TODO #[pyo3_opentelemetry::pypropagate(on_context_extraction_failure="ignore")]
        async fn measure_expectation(
            request: http::ExpectationRequest,
            client: PyQvmClient,
            options: Option<QvmOptions>) -> PyResult<Vec<f64>> {
            client
                .measure_expectation(&request, &options.unwrap_or_default())
                .await
                .map_err(Into::into)
        }
    }

    py_function_sync_async! {
        #[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
        #[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk.qvm.api"))]
        #[pyfunction]
        #[pyo3(signature = (request, client, options = None))]
        #[tracing::instrument(skip_all)]
        // TODO #[pyo3_opentelemetry::pypropagate(on_context_extraction_failure="ignore")]
        async fn run_and_measure(
            request: http::MultishotMeasureRequest,
            client: PyQvmClient,
            options: Option<QvmOptions>) -> PyResult<Vec<Vec<i64>>> {
            client
                .run_and_measure(&request, &options.unwrap_or_default())
                .await
                .map_err(Into::into)
        }
    }

    py_function_sync_async! {
        #[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
        #[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk.qvm.api"))]
        #[pyfunction]
        #[pyo3(signature = (request, client, options = None))]
        #[tracing::instrument(skip_all)]
        // TODO #[pyo3_opentelemetry::pypropagate(on_context_extraction_failure="ignore")]
        async fn run(
            request: http::MultishotRequest,
            client: PyQvmClient,
            options: Option<QvmOptions>,
        ) -> PyResult<http::MultishotResponse> {
            client
                .run(&request, &options.unwrap_or_default())
                .await
                .map_err(Into::into)
        }
    }
}
