use std::{num::NonZeroU16, sync::Arc};

use opentelemetry::trace::FutureExt;
use pyo3::{pyclass, FromPyObject};
use qcs::{Error, Executable, ExecutionData, JobHandle, Service};
use rigetti_pyo3::{
    impl_as_mut_for_wrapper, py_async, py_sync, py_wrap_error, py_wrap_simple_enum, py_wrap_type,
    pyo3::{exceptions::PyRuntimeError, pymethods, types::PyDict, Py, PyAny, PyResult, Python},
    wrap_error, PyWrapper, ToPython, ToPythonError,
};
use tokio::sync::Mutex;
use tracing::instrument;

use crate::{
    compiler::quilc::{PyCompilerOpts, PyQuilcClient},
    execution_data::PyExecutionData,
    qpu::{api::PyExecutionOptions, translation::PyTranslationOptions},
};

wrap_error!(RustExecutionError(Error));
py_wrap_error!(
    executable,
    RustExecutionError,
    ExecutionError,
    PyRuntimeError
);

// Because Python is garbage-collected, no lifetimes can be guaranteed except `'static`.
//
// `Arc<Mutex<>>` to work around https://github.com/awestlake87/pyo3-asyncio/issues/50
py_wrap_type! {
    PyExecutable(Arc<Mutex<Executable<'static, 'static>>>) as "Executable";
}

impl_as_mut_for_wrapper!(PyExecutable);

#[pyclass]
#[pyo3(name = "ExeParameter")]
#[derive(FromPyObject)]
pub struct PyParameter {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub index: usize,
    #[pyo3(get, set)]
    pub value: f64,
}

#[pymethods]
impl PyParameter {
    #[new]
    pub fn new(name: String, index: usize, value: f64) -> Self {
        Self { name, index, value }
    }
}

/// Invoke a PyExecutable's inner Executable::method with given arguments,
/// then mapped to `Future<Output = Result<PyExecutionData, ExecutionError>>`
macro_rules! py_executable_data {
    ($self: ident, $method: ident $(, $arg: expr)* $(,)?) => {{
        let arc = $self.as_inner().clone();
        async move {
            arc.lock()
                .await
                .$method($($arg),*)
                .await
                .map(ExecutionData::from)
                .map(PyExecutionData::from)
                .map_err(RustExecutionError::from)
                .map_err(RustExecutionError::to_py_err)
        }.with_current_context()
    }};
}

/// Invoke a PyExecutable's inner Executable::method with given arguments,
/// then mapped to `Future<Output = Result<PyJobHandle, ExecutionError>>`
macro_rules! py_job_handle {
    ($self: ident, $method: ident $(, $arg: expr)* $(,)?) => {{
        let arc = $self.as_inner().clone();
        async move {
            arc.lock()
                .await
                .$method($($arg),*)
                .await
                .map(JobHandle::from)
                .map(PyJobHandle::from)
                .map_err(RustExecutionError::from)
                .map_err(RustExecutionError::to_py_err)
        }
        .with_current_context()
    }};
}

#[pyo3_opentelemetry::pypropagate(exclude(new), on_context_extraction_failure = "ignore")]
#[pymethods]
impl PyExecutable {
    #[new]
    #[pyo3(signature = (
        quil,
        /,
        registers = Vec::new(),
        parameters = Vec::new(),
        shots = None,
        quilc_client = None,
        compiler_options = None,
    ))]
    pub fn new(
        quil: String,
        registers: Vec<String>,
        parameters: Vec<PyParameter>,
        #[pyo3(from_py_with = "crate::from_py::optional_non_zero_u16")] shots: Option<NonZeroU16>,
        quilc_client: Option<PyQuilcClient>,
        compiler_options: Option<PyCompilerOpts>,
    ) -> Self {
        let quilc_client = quilc_client.map(|c| c.inner);
        let mut exe = Executable::from_quil(quil).with_quilc_client(quilc_client);

        for reg in registers {
            exe = exe.read_from(reg);
        }

        for param in parameters {
            exe.with_parameter(param.name, param.index, param.value);
        }

        if let Some(shots) = shots {
            exe = exe.with_shots(shots);
        }

        if let Some(options) = compiler_options {
            exe = exe.compiler_options(options.into_inner());
        }

        Self::from(Arc::new(Mutex::new(exe)))
    }

    #[instrument(skip_all)]
    pub fn execute_on_qvm(
        &self,
        py: Python<'_>,
        client: crate::qvm::PyQvmClient,
    ) -> PyResult<PyExecutionData> {
        py_sync!(py, py_executable_data!(self, execute_on_qvm, &client))
    }

    #[instrument(skip_all)]
    pub fn execute_on_qvm_async<'py>(
        &'py self,
        py: Python<'py>,
        client: crate::qvm::PyQvmClient,
    ) -> PyResult<&PyAny> {
        py_async!(py, py_executable_data!(self, execute_on_qvm, &client))
    }

    #[pyo3(signature = (quantum_processor_id, endpoint_id = None, translation_options = None, execution_options = None))]
    pub fn execute_on_qpu(
        &self,
        py: Python<'_>,
        quantum_processor_id: String,
        endpoint_id: Option<String>,
        translation_options: Option<PyTranslationOptions>,
        execution_options: Option<PyExecutionOptions>,
    ) -> PyResult<PyExecutionData> {
        let translation_options = translation_options.map(|opts| opts.as_inner().clone());
        match endpoint_id {
            Some(endpoint_id) => py_sync!(
                py,
                py_executable_data!(
                    self,
                    execute_on_qpu_with_endpoint,
                    quantum_processor_id,
                    endpoint_id,
                    translation_options,
                )
            ),
            None => py_sync!(
                py,
                py_executable_data!(
                    self,
                    execute_on_qpu,
                    quantum_processor_id,
                    translation_options,
                    execution_options.unwrap_or_default().as_inner(),
                )
            ),
        }
    }

    #[pyo3(signature = (quantum_processor_id, endpoint_id = None, translation_options = None, execution_options = None))]
    pub fn execute_on_qpu_async<'py>(
        &'py self,
        py: Python<'py>,
        quantum_processor_id: String,
        endpoint_id: Option<String>,
        translation_options: Option<PyTranslationOptions>,
        execution_options: Option<PyExecutionOptions>,
    ) -> PyResult<&PyAny> {
        let translation_options = translation_options.map(|opts| opts.as_inner().clone());
        match endpoint_id {
            Some(endpoint_id) => py_async!(
                py,
                py_executable_data!(
                    self,
                    execute_on_qpu_with_endpoint,
                    quantum_processor_id,
                    endpoint_id,
                    translation_options,
                )
            ),
            None => py_async!(
                py,
                py_executable_data!(
                    self,
                    execute_on_qpu,
                    quantum_processor_id,
                    translation_options,
                    execution_options.unwrap_or_default().as_inner(),
                )
            ),
        }
    }

    #[pyo3(signature = (quantum_processor_id, endpoint_id = None, translation_options = None, execution_options = None))]
    pub fn submit_to_qpu(
        &self,
        py: Python<'_>,
        quantum_processor_id: String,
        endpoint_id: Option<String>,
        translation_options: Option<PyTranslationOptions>,
        execution_options: Option<PyExecutionOptions>,
    ) -> PyResult<PyJobHandle> {
        let translation_options = translation_options.map(|opts| opts.as_inner().clone());
        match endpoint_id {
            Some(endpoint_id) => py_sync!(
                py,
                py_job_handle!(
                    self,
                    submit_to_qpu_with_endpoint,
                    quantum_processor_id,
                    endpoint_id,
                    translation_options,
                )
            ),
            None => py_sync!(
                py,
                py_job_handle!(
                    self,
                    submit_to_qpu,
                    quantum_processor_id,
                    translation_options,
                    execution_options.unwrap_or_default().as_inner(),
                )
            ),
        }
    }

    #[pyo3(signature = (quantum_processor_id, endpoint_id = None, translation_options = None, execution_options = None))]
    pub fn submit_to_qpu_async<'py>(
        &'py self,
        py: Python<'py>,
        quantum_processor_id: String,
        endpoint_id: Option<String>,
        translation_options: Option<PyTranslationOptions>,
        execution_options: Option<PyExecutionOptions>,
    ) -> PyResult<&PyAny> {
        let translation_options = translation_options.map(|opts| opts.as_inner().clone());
        match endpoint_id {
            Some(endpoint_id) => {
                py_async!(
                    py,
                    py_job_handle!(
                        self,
                        submit_to_qpu_with_endpoint,
                        quantum_processor_id,
                        endpoint_id,
                        translation_options,
                    )
                )
            }
            None => py_async!(
                py,
                py_job_handle!(
                    self,
                    submit_to_qpu,
                    quantum_processor_id,
                    translation_options,
                    execution_options.unwrap_or_default().as_inner(),
                )
            ),
        }
    }

    pub fn retrieve_results(
        &mut self,
        py: Python<'_>,
        job_handle: PyJobHandle,
    ) -> PyResult<PyExecutionData> {
        py_sync!(
            py,
            py_executable_data!(self, retrieve_results, job_handle.into())
        )
    }

    pub fn retrieve_results_async<'py>(
        &'py mut self,
        py: Python<'py>,
        job_handle: PyJobHandle,
    ) -> PyResult<&PyAny> {
        py_async!(
            py,
            py_executable_data!(self, retrieve_results, job_handle.into())
        )
    }
}

py_wrap_simple_enum! {
    PyService(Service) as "Service" {
        Quilc as Quilc,
        Qvm as QVM,
        Qcs as QCS,
        Qpu as QPU
    }
}

py_wrap_type! {
    PyJobHandle(JobHandle<'static>) as "JobHandle";
}

#[pymethods]
impl PyJobHandle {
    #[getter]
    pub fn job_id(&self) -> String {
        self.as_inner().job_id().to_string()
    }

    #[getter]
    pub fn readout_map(&self, py: Python) -> PyResult<Py<PyDict>> {
        self.as_inner().readout_map().to_python(py)
    }
}
