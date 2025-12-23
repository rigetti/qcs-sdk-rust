//! Running programs on a QPU.
use std::collections::HashMap;
use std::time::Duration;

use numpy::Complex32;
use pyo3::{prelude::*, types::PyTuple};
use rigetti_pyo3::{create_init_submodule, impl_repr, py_function_sync_async};

#[cfg(feature = "stubs")]
use pyo3_stub_gen::derive::{
    gen_stub_pyclass, gen_stub_pyclass_complex_enum, gen_stub_pyfunction, gen_stub_pymethods,
};

use qcs_api_client_grpc::models::controller::{
    data_value, readout_values, ControllerJobExecutionResult,
};

use crate::{
    client::Qcs,
    python::errors,
    qpu::{
        api::{
            self, ApiExecutionOptions, ApiExecutionOptionsBuilder, ApiExecutionOptionsBuilderError,
            ConnectionStrategy, ExecutionOptions, ExecutionOptionsBuilder,
            ExecutionOptionsBuilderError, JobId, QpuApiDuration, QpuApiError,
        },
        result_data::MemoryValues,
    },
};

create_init_submodule! {
    classes: [
        Register,
        ExecutionResult,
        ExecutionResults,
        ExecutionOptions,
        ExecutionOptionsBuilder,
        ApiExecutionOptions,
        ApiExecutionOptionsBuilder,
        PyQpuApiDuration
    ],
    complex_enums: [ ConnectionStrategy ],
    errors: [
        errors::QpuApiError,
        errors::SubmissionError,
        errors::BuildOptionsError
    ],
    funcs: [
        py_submit,
        py_submit_async,
        py_submit_with_parameter_batch,
        py_submit_with_parameter_batch_async,
        py_cancel_job,
        py_cancel_job_async,
        py_cancel_jobs,
        py_cancel_jobs_async,
        py_retrieve_results,
        py_retrieve_results_async
    ],
}

impl_repr!(ExecutionOptions);
impl_repr!(ApiExecutionOptions);
impl_repr!(ConnectionStrategy);

#[derive(Clone)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(name = "ExecutionOptionsBuilder", module = "qcs_sdk.qpu.api")]
struct PyExecutionOptionsBuilder(ExecutionOptionsBuilder);

/// Variants of data vectors within a single `ExecutionResult`.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass_complex_enum)]
#[pyclass(module = "qcs_sdk.qpu.api")]
pub enum Register {
    I32(Vec<i32>),
    Complex32(Vec<Complex32>),
}

/// The execution readout data from a particular memory location.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(module = "qcs_sdk.qpu.api", frozen, get_all)]
pub struct ExecutionResult {
    /// Describes result shape dimensions.
    pub shape: [usize; 2],
    /// Register data for this result.
    pub data: Register,
    /// Name of the data type.
    pub dtype: String,
}

impl From<readout_values::Values> for ExecutionResult {
    fn from(values: readout_values::Values) -> Self {
        match values {
            readout_values::Values::ComplexValues(cs) => ExecutionResult {
                shape: [cs.values.len(), 1],
                dtype: "complex".into(),
                data: Register::Complex32(
                    cs.values
                        .into_iter()
                        .map(|c| Complex32::new(c.real, c.imaginary))
                        .collect(),
                ),
            },
            readout_values::Values::IntegerValues(ns) => ExecutionResult {
                shape: [ns.values.len(), 1],
                dtype: "integer".into(),
                data: Register::I32(ns.values),
            },
        }
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl ExecutionResult {
    #[staticmethod]
    fn from_register(register: Register) -> Self {
        match register {
            Register::I32(ref values) => ExecutionResult {
                shape: [values.len(), 1],
                dtype: "integer".into(),
                data: register,
            },
            Register::Complex32(ref values) => ExecutionResult {
                shape: [values.len(), 1],
                dtype: "complex".into(),
                data: register,
            },
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(module = "qcs_sdk.qpu.api", frozen, get_all)]
pub struct ExecutionResults {
    /// Result data buffers keyed by readout alias name.
    pub buffers: HashMap<String, ExecutionResult>,
    /// QPU execution duration.
    pub execution_duration_microseconds: Option<u64>,
    pub memory: HashMap<String, MemoryValues>,
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl ExecutionResults {
    #[new]
    fn __new__(
        buffers: HashMap<String, ExecutionResult>,
        memory: HashMap<String, MemoryValues>,
        execution_duration_microseconds: Option<u64>,
    ) -> Self {
        Self {
            buffers,
            execution_duration_microseconds,
            memory,
        }
    }
}

impl From<data_value::Value> for MemoryValues {
    fn from(value: data_value::Value) -> Self {
        match value {
            data_value::Value::Binary(value) => MemoryValues::Binary(value.data),
            data_value::Value::Integer(value) => MemoryValues::Integer(value.data),
            data_value::Value::Real(value) => MemoryValues::Real(value.data),
        }
    }
}

impl From<ControllerJobExecutionResult> for ExecutionResults {
    fn from(result: ControllerJobExecutionResult) -> Self {
        let buffers = result
            .readout_values
            .into_iter()
            .filter_map(|(key, val)| val.values.map(|values| (key, values.into())))
            .collect();

        let memory = result
            .memory_values
            .into_iter()
            .filter_map(|(key, value)| value.value.map(|value| (key, value.into())))
            .collect();

        Self {
            buffers,
            execution_duration_microseconds: Some(result.execution_duration_microseconds),
            memory,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(name = "QpuApiDuration", module = "qcs_sdk.qpu.api", frozen)]
pub struct PyQpuApiDuration {
    inner: QpuApiDuration,
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyQpuApiDuration {
    #[new]
    fn __new__(seconds: i64, nanos: i32) -> Self {
        Self {
            inner: QpuApiDuration { seconds, nanos },
        }
    }

    #[getter]
    fn seconds(&self) -> i64 {
        self.inner.seconds
    }

    #[getter]
    fn nanos(&self) -> i32 {
        self.inner.nanos
    }
}

/// Errors that may occur when submitting a program for execution.
#[derive(Debug, thiserror::Error)]
pub enum SubmissionError {
    /// An API error occurred
    #[error("An API error occurred: {0}")]
    QpuApiError(#[from] QpuApiError),

    /// Job could not be deserialized
    #[error("Failed to deserialize job: {0}")]
    DeserializeError(#[from] serde_json::Error),
}

/// Errors that may occur when building execution options.
#[derive(Debug, thiserror::Error)]
pub enum BuildOptionsError {
    /// An error occurred building execution options.
    #[error("An error occurred building execution options: {0}")]
    ExecutionOptions(#[from] ExecutionOptionsBuilderError),

    /// An error occurred building API execution options.
    #[error("An error occurred building API execution options: {0}")]
    ApiExecutionOptions(#[from] ApiExecutionOptionsBuilderError),
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl ApiExecutionOptionsBuilder {
    #[new]
    fn __new__() -> Self {
        Self::default()
    }

    #[staticmethod]
    #[pyo3(name = "default")]
    fn py_default() -> Self {
        Self::default()
    }

    #[setter(bypass_settings_protection)]
    fn py_bypass_settings_protection(&mut self, bypass_settings_protection: bool) {
        self.bypass_settings_protection(bypass_settings_protection);
    }

    #[setter(timeout)]
    fn py_timeout(&mut self, timeout: Option<PyQpuApiDuration>) {
        self.timeout(timeout.map(|timeout| timeout.inner));
    }

    #[pyo3(name = "build")]
    fn py_build(&self) -> Result<ApiExecutionOptions, BuildOptionsError> {
        Ok(self.build()?)
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl ApiExecutionOptions {
    #[staticmethod]
    #[pyo3(name = "default")]
    fn py_default() -> Self {
        Self::default()
    }

    /// Get the configured `timeout` value.
    ///
    /// Note, this is the timeout while running a job; the job will be evicted from
    /// the hardware once this time has elapsed.
    ///
    /// If unset, the job's estimated duration will be used;
    /// if the job does not have an estimated duration, the default
    /// timeout is selected by the service.
    ///
    /// The service may also enforce a maximum value for this field.
    #[getter(timeout)]
    fn py_timeout(&self) -> Option<PyQpuApiDuration> {
        self.inner.timeout.map(|inner| PyQpuApiDuration { inner })
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl ExecutionOptions {
    #[staticmethod]
    #[pyo3(name = "default")]
    fn py_default() -> Self {
        Self::default()
    }

    #[staticmethod]
    #[pyo3(name = "builder")]
    fn py_builder() -> ExecutionOptionsBuilder {
        ExecutionOptionsBuilder::default()
    }

    #[getter]
    fn timeout_seconds(&self) -> Option<f64> {
        self.timeout.map(|timeout| timeout.as_secs_f64())
    }

    #[getter(api_options)]
    fn py_api_options(&self) -> Option<ApiExecutionOptions> {
        self.api_options.map(Into::into)
    }

    #[new]
    #[pyo3(signature = (
            connection_strategy = ConnectionStrategy::default(),
            timeout = Some(Duration::from_secs(30)),
            api_options = None
    ))]
    fn __new__(
        connection_strategy: ConnectionStrategy,
        timeout: Option<Duration>,
        api_options: Option<ApiExecutionOptions>,
    ) -> Self {
        Self {
            connection_strategy,
            timeout,
            api_options: api_options.map(Into::into),
        }
    }

    fn __getnewargs__(
        &self,
    ) -> (
        ConnectionStrategy,
        Option<Duration>,
        Option<ApiExecutionOptions>,
    ) {
        (
            self.connection_strategy.clone(),
            self.timeout,
            self.api_options.map(Into::into),
        )
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl ExecutionOptionsBuilder {
    #[new]
    fn __new__() -> Self {
        Self::default()
    }

    #[staticmethod]
    #[pyo3(name = "default")]
    fn py_default() -> Self {
        Self::default()
    }

    #[setter(connection_strategy)]
    fn py_connection_strategy(&mut self, connection_strategy: ConnectionStrategy) {
        self.connection_strategy(connection_strategy);
    }

    #[setter(api_options)]
    fn py_api_options(&mut self, api_options: Option<ApiExecutionOptions>) {
        self.api_options(api_options.map(Into::into));
    }

    #[setter(timeout_seconds)]
    fn py_timeout_seconds(&mut self, timeout_seconds: Option<f64>) {
        self.timeout(timeout_seconds.map(Duration::from_secs_f64));
    }

    #[pyo3(name = "build")]
    fn py_build(&self) -> Result<ExecutionOptions, BuildOptionsError> {
        Ok(self.build()?)
    }
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl ConnectionStrategy {
    #[staticmethod]
    #[pyo3(name = "default")]
    fn py_default() -> Self {
        Self::default()
    }

    fn get_endpoint_id(&self) -> PyResult<String> {
        match self {
            ConnectionStrategy::EndpointId(id) => Ok(id.clone()),
            _ => Err(errors::QpuApiError::new_err(
                "ConnectionStrategy is not an EndpointId",
            )),
        }
    }

    #[gen_stub(override_return_type(type_repr = "tuple[str] | tuple[()]"))]
    fn __getnewargs__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        match self {
            Self::EndpointId(id) => (id.clone(),).into_pyobject(py),
            Self::Gateway() | Self::DirectAccess() => Ok(PyTuple::empty(py)),
        }
    }
}

#[cfg(feature = "stubs")]
mod stubs {
    use super::{ApiExecutionOptionsBuilder, ExecutionOptionsBuilder, JobId};
    use pyo3_stub_gen::{PyStubType, TypeInfo};

    impl PyStubType for JobId {
        fn type_output() -> TypeInfo {
            TypeInfo::builtin("str")
        }
    }

    impl PyStubType for &mut ExecutionOptionsBuilder {
        fn type_output() -> TypeInfo {
            ExecutionOptionsBuilder::type_output()
        }
    }

    impl PyStubType for &mut ApiExecutionOptionsBuilder {
        fn type_output() -> TypeInfo {
            ApiExecutionOptionsBuilder::type_output()
        }
    }
}

py_function_sync_async! {
    /// Submits an executable `program` to be run on the specified QPU
    ///
    /// # Errors
    ///
    /// May return an error if
    /// * an engagement is not available
    /// * an RPCQ client cannot be built
    /// * the program cannot be submitted
    #[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk.qpu.api"))]
    #[pyfunction]
    #[pyo3(signature = (program, patch_values, quantum_processor_id = None, client = None, execution_options = None))]
    #[pyo3_opentelemetry::pypropagate(on_context_extraction_failure="ignore")]
    async fn submit(
        program: String,
        patch_values: HashMap<String, Vec<f64>>,
        quantum_processor_id: Option<String>,
        client: Option<Qcs>,
        execution_options: Option<ExecutionOptions>,
    ) -> PyResult<JobId> {
        // TODO: Open an issue for this.
        //
        // Is there a better way to map these patch_values keys? This
        // negates the whole purpose of [`submit`] using `Box<str>`,
        // instead of `String` directly, which normally would decrease
        // copies _and_ require less space, since str can't be extended.
        //
        // Even better: this eventually makes it way to
        // `params_into_job_execution_configuration`,
        // which converts it to `HashMap<String, Vec<f64>>`!
        let patch_values = patch_values
            .into_iter()
            .map(|(k, v)| (k.into_boxed_str(), v))
            .collect();

        let job = serde_json::from_str(&program)
            .map_err(SubmissionError::from)?;

        api::submit(
            quantum_processor_id.as_deref(),
            job,
            &patch_values,
            &client.unwrap_or_else(Qcs::load),
            &execution_options.unwrap_or_default()
        )
        .await
        .map_err(Into::into)
    }
}

py_function_sync_async! {
    #[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk.qpu.api"))]
    #[pyfunction]
    #[pyo3(signature = (program, patch_values, quantum_processor_id = None, client = None, execution_options = None))]
    #[pyo3_opentelemetry::pypropagate(on_context_extraction_failure="ignore")]
    async fn submit_with_parameter_batch(
        program: String,
        patch_values: Vec<HashMap<String, Vec<f64>>>,
        quantum_processor_id: Option<String>,
        client: Option<Qcs>,
        execution_options: Option<ExecutionOptions>,
    ) -> PyResult<Vec<JobId>> {
        let patch_values = patch_values
            .into_iter()
            .map(|m| m
                .into_iter()
                .map(|(k, v)| (k.into_boxed_str(), v))
                .collect()
            ).collect::<Vec<_>>();

        let job = serde_json::from_str(&program)
            .map_err(SubmissionError::from)?;

        api::submit_with_parameter_batch(
            quantum_processor_id.as_deref(),
            job,
            &patch_values,
            &client.unwrap_or_else(Qcs::load),
            &execution_options.unwrap_or_default()
        )
        .await
        .map_err(Into::into)
    }
}

py_function_sync_async! {
    #[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk.qpu.api"))]
    #[pyfunction]
    #[pyo3(signature = (job_ids, quantum_processor_id = None, client = None, execution_options = None))]
    async fn cancel_jobs(
        job_ids: Vec<JobId>,
        quantum_processor_id: Option<String>,
        client: Option<Qcs>,
        execution_options: Option<ExecutionOptions>,
    ) -> PyResult<()> {
        api::cancel_jobs(
            job_ids,
            quantum_processor_id.as_deref(),
            &client.unwrap_or_else(Qcs::load),
            &execution_options.unwrap_or_default()
        )
        .await
        .map_err(Into::into)
    }
}

py_function_sync_async! {
    #[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk.qpu.api"))]
    #[pyfunction]
    #[pyo3(signature = (job_id, quantum_processor_id = None, client = None, execution_options = None))]
    async fn cancel_job(
        job_id: JobId,
        quantum_processor_id: Option<String>,
        client: Option<Qcs>,
        execution_options: Option<ExecutionOptions>,
    ) -> PyResult<()> {
        api::cancel_job(
            job_id,
            quantum_processor_id.as_deref(),
            &client.unwrap_or_else(Qcs::load),
            &execution_options.unwrap_or_default()
        )
        .await
        .map_err(Into::into)
    }
}

py_function_sync_async! {
    #[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk.qpu.api"))]
    #[pyfunction]
    #[pyo3(signature = (job_id, quantum_processor_id = None, client = None, execution_options = None))]
    #[pyo3_opentelemetry::pypropagate(on_context_extraction_failure="ignore")]
    async fn retrieve_results(
        job_id: JobId,
        quantum_processor_id: Option<String>,
        client: Option<Qcs>,
        execution_options: Option<ExecutionOptions>
    ) -> PyResult<ExecutionResults> {
        api::retrieve_results(
            job_id,
            quantum_processor_id.as_deref(),
            &client.unwrap_or_else(Qcs::load),
            &execution_options.unwrap_or_default()
        )
        .await
        .map(Into::into)
        .map_err(Into::into)
    }
}
