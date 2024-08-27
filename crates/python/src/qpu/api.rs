//! Running programs on a QPU.
use std::collections::HashMap;
use std::time::Duration;

use numpy::{ndarray::Array2, Complex32, PyArray2};
use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    pyclass,
    pyclass::CompareOp,
    pyfunction, pymethods,
    types::{PyComplex, PyDict, PyInt, PyTuple},
    IntoPy, Py, PyObject, PyResult, Python, ToPyObject,
};
use qcs::qpu::api::{
    ApiExecutionOptions, ApiExecutionOptionsBuilder, ConnectionStrategy, ExecutionOptions,
    ExecutionOptionsBuilder,
};
use qcs_api_client_grpc::models::controller::{
    data_value, readout_values, ControllerJobExecutionResult,
};
use rigetti_pyo3::{
    create_init_submodule, impl_as_mut_for_wrapper, impl_repr, num_complex, py_wrap_error,
    py_wrap_type, py_wrap_union_enum, wrap_error, PyWrapper, ToPythonError,
};

use crate::py_sync::py_function_sync_async;

use crate::client::PyQcsClient;

use super::result_data::PyMemoryValues;

create_init_submodule! {
    classes: [
        PyRegister,
        ExecutionResult,
        ExecutionResults,
        PyConnectionStrategy,
        PyExecutionOptions,
        PyExecutionOptionsBuilder,
        PyApiExecutionOptions,
        PyApiExecutionOptionsBuilder
    ],
    errors: [
        SubmissionError,
        QpuApiError
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
    #[pyo3_opentelemetry::pypropagate(on_context_extraction_failure="ignore")]
    #[pyfunction]
    #[pyo3(signature = (program, patch_values, quantum_processor_id = None, client = None, execution_options = None))]
    async fn submit(
        program: String,
        patch_values: HashMap<String, Vec<f64>>,
        quantum_processor_id: Option<String>,
        client: Option<PyQcsClient>,
        execution_options: Option<PyExecutionOptions>,
    ) -> PyResult<String> {
        let client = PyQcsClient::get_or_create_client(client);

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

        let job_id = qcs::qpu::api::submit(quantum_processor_id.as_deref(), job, &patch_values, &client, execution_options.unwrap_or_default().as_inner()).await
            .map_err(RustSubmissionError::from)
            .map_err(RustSubmissionError::to_py_err)?;

        Ok(job_id.to_string())
    }
}

py_function_sync_async! {
    #[pyo3_opentelemetry::pypropagate(on_context_extraction_failure="ignore")]
    #[pyfunction]
    #[pyo3(signature = (program, patch_values, quantum_processor_id = None, client = None, execution_options = None))]
    async fn submit_with_parameter_batch(
        program: String,
        patch_values: Vec<HashMap<String, Vec<f64>>>,
        quantum_processor_id: Option<String>,
        client: Option<PyQcsClient>,
        execution_options: Option<PyExecutionOptions>,
    ) -> PyResult<Vec<String>> {
        let client = PyQcsClient::get_or_create_client(client);

        let patch_values: Vec<HashMap<Box<str>, Vec<f64>>> = patch_values
            .into_iter()
            .map(|m| m.into_iter().map(|(k, v)| (k.into_boxed_str(), v)).collect())
            .collect();

        let job = serde_json::from_str(&program)
            .map_err(RustSubmissionError::from)
            .map_err(RustSubmissionError::to_py_err)?;

        Ok(qcs::qpu::api::submit_with_parameter_batch(quantum_processor_id.as_deref(), job, &patch_values, &client, execution_options.unwrap_or_default().as_inner()).await
            .map_err(RustSubmissionError::from)
            .map_err(RustSubmissionError::to_py_err)?
            .into_iter()
            .map(|id| id.to_string())
            .collect())
    }
}

wrap_error!(RustQpuApiError(qcs::qpu::api::QpuApiError));
py_wrap_error!(runner, RustQpuApiError, QpuApiError, PyRuntimeError);

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

/// TODO: Remove new
#[pyclass]
#[derive(Clone, Debug)]
pub struct ExecutionResultNew {
    raw_job_result: ControllerJobExecutionResult,
    #[pyo3(get, set)]
    pub readout_mappings: HashMap<String, String>,
}

impl From<ControllerJobExecutionResult> for ExecutionResultNew {
    fn from(raw_job_result: ControllerJobExecutionResult) -> Self {
        Self {
            raw_job_result,
            readout_mappings: HashMap::new(),
        }
    }
}

#[pymethods]
impl ExecutionResultNew {
    #[new]
    fn new() -> Self {
        Self {
            raw_job_result: ControllerJobExecutionResult::default(),
            readout_mappings: HashMap::new(),
        }
    }

    fn get_readout_values(
        &self,
        py: Python<'_>,
        readout_mappings: Option<HashMap<String, String>>,
    ) -> PyResult<(Py<PyArray2<f64>>, Py<PyDict>)> {
        let mappings = match &readout_mappings {
            Some(mappings) => mappings,
            None => &self.readout_mappings,
        };

        let job = &self.raw_job_result;
        let mut all_values = Vec::new();
        let mut views = HashMap::new();
        let mut current_index = 0;

        for (key, readout) in &job.readout_values {
            let mapped_key = mappings.get(key).unwrap_or(key).to_string();
            let (view_start, view_length) = match &readout.values {
                Some(readout_values::Values::IntegerValues(values)) => {
                    all_values.extend(values.values.iter().map(|&v| v as f64));
                    let start = current_index;
                    let length = values.values.len();
                    current_index += length;
                    (start, length)
                }
                Some(readout_values::Values::ComplexValues(values)) => {
                    all_values.extend(
                        values
                            .values
                            .iter()
                            .flat_map(|c| vec![c.real as f64, c.imaginary as f64]),
                    );
                    let start = current_index;
                    let length = values.values.len();
                    current_index += length;
                    (start, length)
                }
                None => continue,
            };

            // Store slice info for later view creation
            views.insert(mapped_key, (view_start, view_length));
        }

        // Create the ndarray array
        let num_rows = all_values.len();
        let array_shape = (num_rows, 2); // Assuming 2 columns for f64 data
        let ndarray_array = Array2::from_shape_vec(array_shape, all_values)
            .map_err(|_| PyValueError::new_err("boom"))?;

        // Convert ndarray to PyArray2
        let py_array = PyArray2::from_array(py, &ndarray_array);

        // Execute Python code to create views
        let py_views = PyDict::new(py);

        for (key, (start, length)) in views {
            let view_code = format!(
                "import numpy as np; array = np.asarray(array); array[{}:{}]",
                start,
                start + length
            );
            let view: PyObject = py
                .eval(
                    &view_code,
                    Some(PyDict::new(py)),
                    Some(PyDict::from_sequence(
                        py,
                        [("array", py_array)].to_object(py),
                    )?),
                )?
                .into();
            py_views.set_item(key, view)?;
        }

        Ok((py_array.into(), py_views.into()))
    }

    fn get_memory_values(&self) -> PyResult<PyArray2<PyObject>> {
        todo!()
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

#[pymethods]
impl ExecutionResult {
    #[staticmethod]
    fn from_register(register: PyRegister) -> Self {
        match register.as_inner() {
            Register::I32(values) => ExecutionResult {
                shape: [values.len(), 1],
                dtype: "integer".into(),
                data: register,
            },
            Register::Complex32(values) => ExecutionResult {
                shape: [values.len(), 1],
                dtype: "complex".into(),
                data: register,
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
    #[pyo3(get)]
    pub memory: HashMap<String, PyMemoryValues>,
}

#[pymethods]
impl ExecutionResults {
    #[new]
    fn new(
        buffers: HashMap<String, ExecutionResult>,
        memory: HashMap<String, PyMemoryValues>,
        execution_duration_microseconds: Option<u64>,
    ) -> Self {
        Self {
            buffers,
            execution_duration_microseconds,
            memory,
        }
    }
}

impl ExecutionResults {
    fn from_controller_job_execution_result(
        py: Python<'_>,
        result: ControllerJobExecutionResult,
    ) -> PyResult<Self> {
        let buffers = result
            .readout_values
            .into_iter()
            .filter_map(|(key, val)| val.values.map(|values| (key, values.into())))
            .collect();

        let memory = result.memory_values.iter().try_fold(
            HashMap::with_capacity(result.memory_values.len()),
            |mut acc, (key, value)| -> PyResult<HashMap<_, _>> {
                if let Some(value) = &value.value {
                    acc.insert(
                        key.clone(),
                        match value {
                            data_value::Value::Binary(value) => PyMemoryValues::from_binary(
                                py,
                                value
                                    .data
                                    .iter()
                                    .map(|v| v.to_object(py).extract(py))
                                    .collect::<PyResult<Vec<_>>>()?,
                            )?,
                            data_value::Value::Integer(value) => PyMemoryValues::from_integer(
                                py,
                                value
                                    .data
                                    .iter()
                                    .map(|v| v.to_object(py).extract(py))
                                    .collect::<PyResult<Vec<_>>>()?,
                            )?,
                            data_value::Value::Real(value) => PyMemoryValues::from_real(
                                py,
                                value
                                    .data
                                    .iter()
                                    .map(|v| v.to_object(py).extract(py))
                                    .collect::<PyResult<Vec<_>>>()?,
                            )?,
                        },
                    )
                } else {
                    None
                };

                Ok(acc)
            },
        )?;

        Ok(Self {
            buffers,
            execution_duration_microseconds: Some(result.execution_duration_microseconds),
            memory,
        })
    }
}

py_function_sync_async! {
    #[pyfunction]
    #[pyo3(signature = (job_ids, quantum_processor_id = None, client = None, execution_options = None))]
    async fn cancel_jobs(
        job_ids: Vec<String>,
        quantum_processor_id: Option<String>,
        client: Option<PyQcsClient>,
        execution_options: Option<PyExecutionOptions>,
    ) -> PyResult<()> {
        let client = PyQcsClient::get_or_create_client(client);

        qcs::qpu::api::cancel_jobs(
            job_ids.into_iter().map(|id| id.into()).collect(),
            quantum_processor_id.as_deref(),
            &client,
            execution_options.unwrap_or_default().as_inner()
        )
        .await
        .map_err(RustQpuApiError::from).map_err(RustQpuApiError::to_py_err)?;

        Ok(())
    }
}

py_function_sync_async! {
    #[pyfunction]
    #[pyo3(signature = (job_id, quantum_processor_id = None, client = None, execution_options = None))]
    async fn cancel_job(
        job_id: String,
        quantum_processor_id: Option<String>,
        client: Option<PyQcsClient>,
        execution_options: Option<PyExecutionOptions>,
    ) -> PyResult<()> {
        let client = PyQcsClient::get_or_create_client(client);

        qcs::qpu::api::cancel_job(
            job_id.into(),
            quantum_processor_id.as_deref(),
            &client,
            execution_options.unwrap_or_default().as_inner()
        )
        .await
        .map_err(RustQpuApiError::from).map_err(RustQpuApiError::to_py_err)?;

        Ok(())
    }
}

py_function_sync_async! {
    #[pyo3_opentelemetry::pypropagate(on_context_extraction_failure="ignore")]
    #[pyfunction]
    #[pyo3(signature = (job_id, quantum_processor_id = None, client = None, execution_options = None))]
    async fn retrieve_results(
        job_id: String,
        quantum_processor_id: Option<String>,
        client: Option<PyQcsClient>,
        execution_options: Option<PyExecutionOptions>
    ) -> PyResult<ExecutionResults> {
        let client = PyQcsClient::get_or_create_client(client);

        let results = qcs::qpu::api::retrieve_results(job_id.into(), quantum_processor_id.as_deref(), &client, execution_options.unwrap_or_default().as_inner())
            .await
            .map_err(RustQpuApiError::from)
            .map_err(RustQpuApiError::to_py_err)?;

        Python::with_gil(|py| {
            ExecutionResults::from_controller_job_execution_result(py, results)
        })
    }
}

py_wrap_type! {
    #[derive(Debug, Default)]
    #[pyo3(module = "qcs_sdk.qpu.api")]
    PyExecutionOptions(ExecutionOptions) as "ExecutionOptions"
}
impl_repr!(PyExecutionOptions);
impl_as_mut_for_wrapper!(PyExecutionOptions);

py_wrap_type! {
    #[derive(Debug, Default)]
    PyApiExecutionOptions(ApiExecutionOptions) as "APIExecutionOptions"
}
impl_repr!(PyApiExecutionOptions);
impl_as_mut_for_wrapper!(PyApiExecutionOptions);

#[pymethods]
impl PyExecutionOptions {
    #[staticmethod]
    fn default() -> Self {
        Self::from(ExecutionOptions::default())
    }

    #[staticmethod]
    fn builder() -> PyExecutionOptionsBuilder {
        PyExecutionOptionsBuilder::default()
    }

    #[getter]
    fn connection_strategy(&self) -> PyConnectionStrategy {
        PyConnectionStrategy(self.as_inner().connection_strategy().clone())
    }

    #[getter]
    fn timeout_seconds(&self) -> Option<f64> {
        self.as_inner()
            .timeout()
            .map(|timeout| timeout.as_secs_f64())
    }

    #[getter]
    fn api_options(&self) -> Option<PyApiExecutionOptions> {
        self.as_inner()
            .api_options()
            .map(|x| PyApiExecutionOptions(x.clone().into()))
    }

    fn __richcmp__(&self, py: Python<'_>, other: &Self, op: CompareOp) -> PyObject {
        match op {
            CompareOp::Eq => (self.as_inner() == other.as_inner()).into_py(py),
            _ => py.NotImplemented(),
        }
    }

    fn __reduce__<'py>(&mut self, py: Python<'py>) -> PyResult<&'py PyTuple> {
        let callable = py.get_type::<Self>().getattr("_from_parts")?;
        Ok(PyTuple::new(
            py,
            [
                callable,
                PyTuple::new(
                    py,
                    &[
                        self.connection_strategy().into_py(py),
                        self.timeout_seconds().into_py(py),
                        self.api_options().into_py(py),
                    ],
                ),
            ],
        ))
    }

    #[staticmethod]
    fn _from_parts(
        connection_strategy: PyConnectionStrategy,
        timeout_seconds: Option<f64>,
        api_options: Option<PyApiExecutionOptions>,
    ) -> PyResult<Self> {
        let mut builder = Self::builder();
        builder.connection_strategy(connection_strategy);
        builder.timeout_seconds(timeout_seconds);
        builder.api_options(api_options);
        builder.build()
    }
}

#[pymethods]
impl PyApiExecutionOptions {
    #[staticmethod]
    fn default() -> Self {
        Self::from(ApiExecutionOptions::default())
    }

    #[staticmethod]
    fn builder() -> PyApiExecutionOptionsBuilder {
        PyApiExecutionOptionsBuilder::default()
    }

    #[getter]
    fn bypass_settings_protection(&self) -> bool {
        self.as_inner().bypass_settings_protection()
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

    #[setter]
    fn connection_strategy(&mut self, connection_strategy: PyConnectionStrategy) {
        // `derive_builder::Builder` doesn't implement AsMut, meaning we can't use `PyWrapperMut`,
        // which forces us into this awkward clone.

        *self = Self::from(
            self.as_inner()
                .clone()
                .connection_strategy(connection_strategy.as_inner().clone())
                .clone(),
        );
    }

    #[setter]
    fn api_options(&mut self, api_options: Option<PyApiExecutionOptions>) {
        *self = Self::from(
            self.as_inner()
                .clone()
                .api_options(api_options.map(|x| x.into_inner().into()))
                .clone(),
        );
    }

    #[setter]
    fn timeout_seconds(&mut self, timeout_seconds: Option<f64>) {
        let timeout = timeout_seconds.map(Duration::from_secs_f64);
        *self = Self::from(self.as_inner().clone().timeout(timeout).clone());
    }

    fn build(&self) -> PyResult<PyExecutionOptions> {
        Ok(PyExecutionOptions::from(
            self.as_inner()
                .build()
                .map_err(|err| PyValueError::new_err(err.to_string()))?,
        ))
    }
}

py_wrap_type! {
    PyApiExecutionOptionsBuilder(ApiExecutionOptionsBuilder) as "APIExecutionOptionsBuilder"
}

#[pymethods]
impl PyApiExecutionOptionsBuilder {
    #[new]
    fn new() -> Self {
        Self::default()
    }

    #[staticmethod]
    fn default() -> Self {
        Self::from(ApiExecutionOptionsBuilder::default())
    }

    #[setter]
    fn bypass_settings_protection(&mut self, bypass_settings_protection: bool) {
        // `derive_builder::Builder` doesn't implement AsMut, meaning we can't use `PyWrapperMut`,
        // which forces us into this awkward clone.

        *self = Self::from(
            self.as_inner()
                .clone()
                .bypass_settings_protection(bypass_settings_protection)
                .clone(),
        );
    }

    fn build(&self) -> PyResult<PyApiExecutionOptions> {
        Ok(PyApiExecutionOptions::from(
            self.as_inner()
                .build()
                .map_err(|err| PyValueError::new_err(err.to_string()))?,
        ))
    }
}

py_wrap_type! {
    #[derive(Default)]
    #[pyo3(module = "qcs_sdk.qpu.api")]
    PyConnectionStrategy(ConnectionStrategy) as "ConnectionStrategy"
}
impl_repr!(PyConnectionStrategy);

#[pymethods]
impl PyConnectionStrategy {
    #[staticmethod]
    #[pyo3(name = "default")]
    fn py_default() -> Self {
        Self::default()
    }

    #[staticmethod]
    fn gateway() -> Self {
        Self(ConnectionStrategy::Gateway)
    }

    #[staticmethod]
    fn direct_access() -> Self {
        Self(ConnectionStrategy::DirectAccess)
    }

    #[staticmethod]
    fn endpoint_id(endpoint_id: String) -> PyResult<Self> {
        Ok(Self(ConnectionStrategy::EndpointId(endpoint_id)))
    }

    fn __richcmp__(&self, py: Python<'_>, other: &Self, op: CompareOp) -> PyObject {
        match op {
            CompareOp::Eq => (self.as_inner() == other.as_inner()).into_py(py),
            _ => py.NotImplemented(),
        }
    }

    fn __reduce__(&self, py: Python<'_>) -> PyResult<PyObject> {
        Ok(match self.as_inner() {
            ConnectionStrategy::Gateway => PyTuple::new(
                py,
                &[
                    py.get_type::<Self>().getattr("gateway")?.to_object(py),
                    PyTuple::empty(py).to_object(py),
                ],
            )
            .to_object(py),
            ConnectionStrategy::DirectAccess => PyTuple::new(
                py,
                &[
                    py.get_type::<Self>()
                        .getattr("direct_access")?
                        .to_object(py),
                    PyTuple::empty(py).to_object(py),
                ],
            )
            .to_object(py),
            ConnectionStrategy::EndpointId(endpoint_id) => PyTuple::new(
                py,
                &[
                    py.get_type::<Self>().getattr("endpoint_id")?.to_object(py),
                    PyTuple::new(py, [endpoint_id]).to_object(py),
                ],
            )
            .to_object(py),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use numpy::PyArray2;
    use pyo3::types::PyDict;
    use qcs_api_client_grpc::models::controller::Complex64;
    use qcs_api_client_grpc::models::controller::Complex64ReadoutValues;
    use qcs_api_client_grpc::models::controller::IntegerReadoutValues;
    use qcs_api_client_grpc::models::controller::ReadoutValues;

    #[test]
    fn test_get_readout_values() {
        // Sample data
        let integer_values = IntegerReadoutValues {
            values: vec![1, 2, 3, 4],
        };

        let complex_values = Complex64ReadoutValues {
            values: vec![
                Complex64 {
                    real: 1.0,
                    imaginary: 2.0,
                },
                Complex64 {
                    real: 3.0,
                    imaginary: 4.0,
                },
            ],
        };

        let mut readout_values = HashMap::new();
        readout_values.insert(
            "int_data".to_string(),
            ReadoutValues {
                values: Some(readout_values::Values::IntegerValues(integer_values)),
            },
        );
        readout_values.insert(
            "complex_data".to_string(),
            ReadoutValues {
                values: Some(readout_values::Values::ComplexValues(complex_values)),
            },
        );

        let job_result = ControllerJobExecutionResult {
            readout_values,
            memory_values: HashMap::new(),
            status: 1,
            status_message: None,
            execution_duration_microseconds: 123456,
        };

        let execution_results = ExecutionResultNew {
            raw_job_result: job_result,
            readout_mappings: HashMap::new(), // No mappings for simplicity
        };

        Python::with_gil(|py| {
            let (py_array, py_views) = execution_results.get_readout_values(py, None).unwrap();

            // Convert PyArray2 to numpy array and verify its contents
            let py_array: &PyArray2<f64> = py_array.as_ref(py);
            let array_data = py_array.readonly().to_owned_array();

            // Expected array data
            let expected_data: Vec<f64> = vec![
                1.0, 0.0, // Integer values
                2.0, 0.0, 3.0, 0.0, 4.0, 0.0, 1.0, 2.0, // Complex values (real, imaginary)
                3.0, 4.0,
            ];

            assert_eq!(array_data.as_slice().unwrap(), &expected_data);

            // Verify views
            let py_views: &PyDict = py_views.as_ref(py);
            let int_view = py_views.get_item("int_data").unwrap().unwrap();
            let complex_view = py_views.get_item("complex_data").unwrap().unwrap();

            // Convert the views to numpy arrays and verify their contents
            let int_view_array: &PyArray2<f64> = int_view.extract().unwrap();
            let complex_view_array: &PyArray2<f64> = complex_view.extract().unwrap();

            let int_view_data = int_view_array.readonly().to_owned_array();
            let complex_view_data = complex_view_array.readonly().to_owned_array();

            // Expected view data
            let expected_int_data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0];
            let expected_complex_data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0];

            assert_eq!(int_view_data.as_slice().unwrap(), &expected_int_data);
            assert_eq!(
                complex_view_data.as_slice().unwrap(),
                &expected_complex_data
            );
        })
    }
}
