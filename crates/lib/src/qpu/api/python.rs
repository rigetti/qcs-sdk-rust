//! Running programs on a QPU.
use std::collections::HashMap;
use std::time::Duration;

use numpy::Complex32;
use pyo3::{prelude::*, types::PyTuple};
use rigetti_pyo3::{create_init_submodule, impl_repr, py_function_sync_async};

#[cfg(feature = "stubs")]
use pyo3_stub_gen::{
    derive::{gen_stub_pyclass, gen_stub_pyfunction, gen_stub_pymethods},
    impl_stub_type,
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

/// Data vectors within a single ``ExecutionResult``.
#[derive(Clone, Debug, IntoPyObject, IntoPyObjectRef, FromPyObject)]
pub enum Register {
    /// A register of 32-bit integers.
    I32(Vec<i32>),
    /// A register of 32-bit complex numbers.
    Complex32(Vec<Complex32>),
}

#[cfg(feature = "stubs")]
impl_stub_type!(Register = Vec<i32> | Vec<Complex32>);

/// Execution readout data from a particular memory location.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(module = "qcs_sdk.qpu.api", frozen, get_all)]
pub struct ExecutionResult {
    /// The shape of the result data.
    pub shape: [usize; 2],
    /// The result data for all shots by the particular memory location.
    pub data: Register,
    /// The type of the result data (as a `numpy` `dtype`).
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
    #[new]
    fn __new__(register: Register) -> Self {
        Self::from_register(register)
    }

    fn __getnewargs__(&self) -> (Register,) {
        (self.data.clone(),)
    }

    /// Build an `ExecutionResult` from a `Register`.
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

/// Execution readout data for all memory locations.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(module = "qcs_sdk.qpu.api", frozen, get_all)]
pub struct ExecutionResults {
    /// The readout results of execution, mapping a published filter node to its data.
    ///
    /// See `TranslationResult.ro_sources` which provides the mapping from the filter node name
    /// to the name of the memory declaration in the source program.
    pub buffers: HashMap<String, ExecutionResult>,
    /// The time spent executing the program.
    pub execution_duration_microseconds: Option<u64>,
    /// The final state of memory for parameters that were read from and written to during
    /// the execution of the program.
    pub memory: HashMap<String, MemoryValues>,
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl ExecutionResults {
    #[new]
    #[pyo3(signature = (buffers, memory, execution_duration_microseconds = None))]
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

/// The duration of an API call.
#[derive(Debug, Clone, Copy)]
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
    #[new]
    #[pyo3(signature = (bypass_settings_protection = false, timeout = None))]
    fn __new__(
        bypass_settings_protection: bool,
        timeout: Option<Duration>,
    ) -> Result<Self, BuildOptionsError> {
        let mut builder = ApiExecutionOptionsBuilder::default();
        builder.bypass_settings_protection(bypass_settings_protection);
        builder.timeout(timeout.map(Into::into));
        Ok(builder.build()?)
    }

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
            Self::EndpointId(id) | Self::EndpointAddress(id) => (id.clone(),).into_pyobject(py),
            Self::Gateway() | Self::DirectAccess() => Ok(PyTuple::empty(py)),
        }
    }
}

#[cfg(feature = "stubs")]
mod stubs {
    use super::{ApiExecutionOptionsBuilder, ExecutionOptionsBuilder, JobId};
    use pyo3::prelude::*;
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

    // The following works around a `mypy` bug that requires write-only properties have getters.
    // These methods won't be available at runtime, but they'll show up in the type stubs,
    // so they're typed in a way that static type checkers should alert the user
    // that they shouldn't be calling the function in question.

    #[doc(hidden)]
    struct WriteOnly(&'static str);

    impl PyStubType for WriteOnly {
        fn type_output() -> TypeInfo {
            TypeInfo::with_module("typing.Never", "typing".into())
        }
    }

    impl<'py> IntoPyObject<'py> for WriteOnly {
        type Target = ();
        type Output = Bound<'py, Self::Target>;
        type Error = PyErr;

        fn into_pyobject(self, _py: Python<'py>) -> Result<Self::Output, Self::Error> {
            Err(pyo3::exceptions::PyAttributeError::new_err(format!(
                "{} is write-only",
                self.0
            )))
        }
    }

    use paste::paste;

    /// Generate stubs for write-only properites.
    ///
    /// # Usage
    ///
    /// For some type `T` with write-only properties `a`, `b`, and `c`, call the macro like so:
    ///
    /// ```ignore
    /// stub_write_only!(T, a, b, c);
    /// ```
    ///
    /// This will generate the correct stubs to satisfy `mypy`
    /// while alerting the user that these properties are write-only.
    macro_rules! stub_write_only {
        ($t:ty, $($field:ident),+ $(,)?) => {
            paste! {
                #[cfg(feature = "stubs")]
                #[pyo3_stub_gen::derive::gen_stub_pymethods]
                #[pymethods]
                impl $t {
                    $(
                        /// DO NOT CALL THIS METHOD.
                        ///
                        /// `mypy` requires write-only properties to have a getter,
                        /// but this method is not actually available at runtime.
                        #[doc(hidden)]
                        #[allow(clippy::unused_self)]
                        #[getter($field)]
                        fn [< get_ $field >](&self) -> WriteOnly {
                            WriteOnly(stringify!($field))
                        }
                    )+
                }
            }
        };
    }

    stub_write_only!(
        ExecutionOptionsBuilder,
        connection_strategy,
        timeout_seconds,
        api_options
    );
    stub_write_only!(
        ApiExecutionOptionsBuilder,
        bypass_settings_protection,
        timeout
    );
}

py_function_sync_async! {
    /// Submits an executable `program` to be run on the specified QPU.
    ///
    /// :param program: An executable program (see ``qcs_sdk.qpu.translation.translate``).
    /// :param patch_values: A mapping of symbols to their desired values (see ``build_patch_values``).
    /// :param quantum_processor_id: The ID of the quantum processor to run the executable on.
    ///     This field is required, unless being used with the ``ConnectionStrategy.endpoint_id()``
    ///     or ``ConnectionStrategy.direct_endpoint_address()`` execution option.
    /// :param client: The ``Qcs`` client to use.
    ///     Creates one using environment configuration if unset.
    ///     See https://docs.rigetti.com/qcs/references/qcs-client-configuration for more information.
    /// :param execution_options: The ``ExecutionOptions`` to use.
    ///     If the connection strategy option used is ``ConnectionStrategy.endpoint_id("endpoint_id")``,
    ///     or ``ConnectionStrategy.direct_endpoint_address("http://some_endpoint_address")``,
    ///     then direct access to "endpoint_id" overrides the ``quantum_processor_id`` parameter.
    ///
    /// :returns: The ID of the submitted job which can be used to fetch results.
    ///
    /// :raises LoadClientError: If there is an issue loading the QCS Client configuration.
    /// :raises SubmissionError: If there was a problem submitting the program for execution.
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
    /// Execute a compiled program on a QPU with multiple sets of ``patch_values``.
    ///
    /// This action is *atomic* in that all jobs will be queued, or none of them will. On success, this
    /// function will return a list of strings where the length and order correspond to the
    /// ``patch_values`` given. However, note that execution in the order of given patch values is not
    /// guaranteed. If there is a failure to queue any of the jobs, then none will be queued.
    ///
    /// :param program: An executable program (see ``translate``).
    /// :param patch_values: An iterable containing one or more mapping of symbols to their desired values.
    /// :param quantum_processor_id: The ID of the quantum processor to run the executable on. This field is required, unless being used with the ``ConnectionStrategy.endpoint_id()`` or ``ConnectionStrategy.direct_endpoint_address()`` execution option.
    /// :param client: The ``Qcs`` client to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration
    /// :param execution_options: The ``ExecutionOptions`` to use.
    ///
    /// :returns: The IDs of the submitted jobs which can be used to fetch results.
    ///
    /// :raises LoadClientError: If there is an issue loading the QCS Client configuration.
    /// :raises SubmissionError: If there was a problem submitting any of the jobs for execution, or if no ``patch_values`` are given.
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
    /// Cancel all given jobs that have yet to begin executing.
    ///
    /// This action is *not* atomic, and will attempt to cancel every job even when some jobs cannot be
    /// cancelled. A job can be cancelled only if it has not yet started executing.
    ///
    /// Success response indicates only that the request was received. Cancellation is not guaranteed,
    /// as it is based on job state at the time of cancellation, and is completed on a best effort
    /// basis.
    ///
    /// :param job_ids: The job IDs to cancel.
    /// :param quantum_processor_id: The quantum processor to execute the job on. This parameter is required unless using the ``ConnectionStrategy.endpoint_id()`` or ``ConnectionStrategy.endpoint_address()`` execution option.
    /// :param client: The ``Qcs`` client to use.
    /// :param execution_options: The ``ExecutionOptions`` to use.
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
    /// Cancel a job that has yet to begin executing.
    ///
    /// This action is *not* atomic, and will attempt to cancel a job even if it cannot be cancelled. A
    /// job can be cancelled only if it has not yet started executing.
    ///
    /// Success response indicates only that the request was received. Cancellation is not guaranteed,
    /// as it is based on job state at the time of cancellation, and is completed on a best effort
    /// basis.
    ///
    /// :param job_id: The job ID to cancel.
    /// :param quantum_processor_id: The quantum processor to execute the job on. This parameter is required unless using the ``ConnectionStrategy.endpoint_id()`` or ``ConnectionStrategy.endpoint_address()`` execution option.
    /// :param client: The ``Qcs`` client to use.
    /// :param execution_options: The ``ExecutionOptions`` to use.
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
    /// Fetches execution results for the given QCS Job ID.
    ///
    /// :param job_id: The ID of the job to retrieve results for.
    /// :param quantum_processor_id: The ID of the quantum processor the job ran on. This field is required, unless being used with the ``ConnectionStrategy.endpoint_id()`` or ``ConnectionStrategy.endpoint_address()`` execution option.
    /// :param client: The ``Qcs`` client to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration
    /// :param execution_options: The ``ExecutionOptions`` to use.
    ///
    /// :returns: Results from execution.
    ///
    /// :raises LoadClientError: If there is an issue loading the QCS Client configuration.
    /// :raises QpuApiError: If there was a problem retrieving the results.
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
