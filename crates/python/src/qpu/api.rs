//! Running programs on a QPU.
use std::collections::HashMap;

use numpy::Complex32;
use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    pyclass, pyfunction, pymethods,
    types::{PyComplex, PyInt, PyString},
    Py, PyResult,
};
use qcs::qpu::api::{ConnectionStrategy, ExecutionOptions, ExecutionOptionsBuilder};
use qcs_api_client_grpc::models::controller::{readout_values, ControllerJobExecutionResult};
use rigetti_pyo3::{
    create_init_submodule, impl_as_mut_for_wrapper, impl_repr, num_complex, py_wrap_error,
    py_wrap_type, py_wrap_union_enum, wrap_error, PyWrapper, ToPythonError,
};

use crate::py_sync::py_function_sync_async;

use crate::client::PyQcsClient;

create_init_submodule! {
    classes: [
        PyRegister,
        ExecutionResult,
        ExecutionResults,
        PyConnectionStrategy,
        PyExecutionOptions,
        PyExecutionOptionsBuilder
    ],
    errors: [
        SubmissionError,
        RetrieveResultsError
    ],
    funcs: [
        py_submit,
        py_submit_async,
        py_retrieve_results,
        py_retrieve_results_async
    ],
}

/// Errors that may occur when submitting a program for execution
#[derive(Debug, thiserror::Error)]
enum RustSubmissionError {
    /// An API error occurred
    #[error("An API error occurred: {0}")]
    QpuApiError(#[from] qcs::qpu::api::QpuApiError),

    /// Job could not be deserialized
    #[error("Failed to deserialize job: {0}")]
    DeserializeError(#[from] serde_json::Error),
}

py_wrap_error!(runner, RustSubmissionError, SubmissionError, PyRuntimeError);

py_function_sync_async! {
    /// Submits an executable `program` to be run on the specified QPU
    ///
    /// # Errors
    ///
    /// May return an error if
    /// * an engagement is not available
    /// * an RPCQ client cannot be built
    /// * the program cannot be submitted
    #[pyfunction(client = "None", endpoint_id = "None")]
    async fn submit(
        program: String,
        patch_values: HashMap<String, Vec<f64>>,
        quantum_processor_id: String,
        client: Option<PyQcsClient>,
        execution_options: Option<PyExecutionOptions>,
    ) -> PyResult<String> {
        let client = PyQcsClient::get_or_create_client(client).await;

        // Is there a better way to map these patch_values keys? This
        // negates the whole purpose of [`submit`] using `Box<str>`,
        // instead of `String` directly, which normally would decrease
        // copies _and_ require less space, since str can't be extended.
        let patch_values = patch_values
            .into_iter()
            .map(|(k, v)| (k.into_boxed_str(), v))
            .collect();

        let job = serde_json::from_str(&program)
            .map_err(RustSubmissionError::from)
            .map_err(RustSubmissionError::to_py_err)?;

        let job_id = qcs::qpu::api::submit(quantum_processor_id, job, &patch_values, &client, execution_options.unwrap_or_default().as_inner()).await
            .map_err(RustSubmissionError::from)
            .map_err(RustSubmissionError::to_py_err)?;

        Ok(job_id.to_string())
    }
}

wrap_error!(RustRetrieveResultsError(qcs::qpu::api::QpuApiError));
py_wrap_error!(
    runner,
    RustRetrieveResultsError,
    RetrieveResultsError,
    PyRuntimeError
);

/// Variants of data vectors within a single ExecutionResult.
#[derive(Clone, Debug)]
pub enum Register {
    I32(Vec<i32>),
    Complex32(Vec<Complex32>),
}

py_wrap_union_enum! {
    #[derive(Debug)]
    PyRegister(Register) as "Register" {
        i32: I32 => Vec<Py<PyInt>>,
        complex32: Complex32 => Vec<Py<PyComplex>>
    }
}

/// The execution readout data from a particular memory location.
#[pyclass]
#[derive(Clone, Debug)]
pub struct ExecutionResult {
    /// Describes result shape dimensions.
    #[pyo3(get)]
    pub shape: [usize; 2],
    /// Register data for this result.
    #[pyo3(get)]
    pub data: PyRegister,
    /// Name of the data type.
    #[pyo3(get)]
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
                        .map(|c| num_complex::Complex32::new(c.real, c.imaginary))
                        .collect(),
                )
                .into(),
            },
            readout_values::Values::IntegerValues(ns) => ExecutionResult {
                shape: [ns.values.len(), 1],
                dtype: "integer".into(),
                data: Register::I32(ns.values).into(),
            },
        }
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct ExecutionResults {
    /// Result data buffers keyed by readout alias name.
    #[pyo3(get)]
    pub buffers: HashMap<String, ExecutionResult>,
    /// QPU execution duration.
    #[pyo3(get)]
    pub execution_duration_microseconds: Option<u64>,
}

impl From<ControllerJobExecutionResult> for ExecutionResults {
    fn from(value: ControllerJobExecutionResult) -> Self {
        let buffers = value
            .readout_values
            .into_iter()
            .filter_map(|(key, val)| val.values.map(|values| (key, values.into())))
            .collect();

        Self {
            buffers,
            execution_duration_microseconds: Some(value.execution_duration_microseconds),
        }
    }
}

py_function_sync_async! {
    #[pyfunction(client = "None", endpoint_id = "None")]
    async fn retrieve_results(
        job_id: String,
        quantum_processor_id: String,
        client: Option<PyQcsClient>,
        execution_options: Option<PyExecutionOptions>
    ) -> PyResult<ExecutionResults> {
        let client = PyQcsClient::get_or_create_client(client).await;

        let results = qcs::qpu::api::retrieve_results(job_id.into(), quantum_processor_id, &client, execution_options.unwrap_or_default().as_inner())
            .await
            .map_err(RustRetrieveResultsError::from)
            .map_err(RustRetrieveResultsError::to_py_err)?;

        Ok(results.into())
    }
}

py_wrap_type! {
    #[derive(Debug, Default)]
    PyExecutionOptions(ExecutionOptions) as "ExecutionOptions"
}
impl_repr!(PyExecutionOptions);
impl_as_mut_for_wrapper!(PyExecutionOptions);

#[pymethods]
impl PyExecutionOptions {
    #[staticmethod]
    fn default() -> Self {
        Self::from(ExecutionOptions::default())
    }
}

py_wrap_type! {
    PyExecutionOptionsBuilder(ExecutionOptionsBuilder) as "ExecutionOptionsBuilder"
}

#[pymethods]
impl PyExecutionOptionsBuilder {
    #[new]
    fn new() -> Self {
        Self::default()
    }

    #[staticmethod]
    fn default() -> Self {
        Self::from(ExecutionOptionsBuilder::default())
    }

    fn connection_strategy(&self, connection_strategy: PyConnectionStrategy) -> Self {
        // `derive_builder::Builder` doesn't implement AsMut, meaning we can't use `PyWrapperMut`,
        // which forces us into this awkward clone.
        Self::from(
            self.as_inner()
                .clone()
                .connection_strategy(connection_strategy.as_inner().clone())
                .clone(),
        )
    }

    fn build(&self) -> PyResult<PyExecutionOptions> {
        Ok(PyExecutionOptions::from(
            self.as_inner()
                .build()
                .map_err(|err| PyValueError::new_err(err.to_string()))?,
        ))
    }
}

py_wrap_union_enum! {
    #[derive(Debug, PartialEq, Eq)]
    PyConnectionStrategy(ConnectionStrategy) as "ConnectionStrategy" {
        gateway: Gateway,
        direct_access: DirectAccess,
        endpoint_id: EndpointId => Py<PyString>
    }
}
impl_repr!(PyConnectionStrategy);

impl Default for PyConnectionStrategy {
    fn default() -> PyConnectionStrategy {
        PyConnectionStrategy::from(ConnectionStrategy::default())
    }
}
