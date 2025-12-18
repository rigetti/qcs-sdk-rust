use std::{collections::HashMap, sync::Arc};

use opentelemetry::trace::FutureExt;
use tokio::sync::Mutex;

use pyo3::prelude::*;
#[cfg(feature = "stubs")]
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use tracing::instrument;

use crate::{
    compiler::{python::PyQuilcClient, quilc::CompilerOpts},
    executable::Executable,
    execution_data::ExecutionData,
    python::{py_sync, NonZeroU16},
    qpu::{
        api::{ExecutionOptions, JobId},
        translation::TranslationOptions,
    },
    qvm::python::PyQvmClient,
    JobHandle,
};

/// A builder interface for executing Quil programs on QVMs and QPUs.
///
/// # Example
///
/// ```python
///
/// PROGRAM = r"""
/// DECLARE ro BIT[2]
///
/// H 0
/// CNOT 0 1
///
/// MEASURE 0 ro[0]
/// MEASURE 1 ro[1]
/// """
///
/// async def run():
///     # TODO: update this example
///     use std::num::NonZeroU16;
///     use qcs::qvm;
///     let qvm_client = qvm::http::HttpClient::from(&Qcs::load());
///     let mut result = Executable::from_quil(PROGRAM).with_qcs_client(Qcs::default()).with_shots(NonZeroU16::new(4).unwrap()).execute_on_qvm(&qvm_client).await.unwrap();
///     // "ro" is the only source read from by default if you don't specify a .read_from()
///
///     // We first convert the readout data to a [`RegisterMap`] to get a mapping of registers
///     // (ie. "ro") to a [`RegisterMatrix`], `M`, where M[`shot`][`index`] is the value for
///     // the memory offset `index` during shot `shot`.
///     // There are some programs where QPU readout data does not fit into a [`RegisterMap`], in
///     // which case you should build the matrix you need from [`QpuResultData`] directly. See
///     // the [`RegisterMap`] documentation for more information on when this transformation
///     // might fail.
///     let data = result.result_data
///                         .to_register_map()
///                         .expect("should convert to readout map")
///                         .get_register_matrix("ro")
///                         .expect("should have data in ro")
///                         .as_integer()
///                         .expect("should be integer matrix")
///                         .to_owned();
///
///     // In this case, we ran the program for 4 shots, so we know the number of rows is 4.
///     assert_eq!(data.nrows(), 4);
///     for shot in data.rows() {
///         // Each shot will contain all the memory, in order, for the vector (or "register") we
///         // requested the results of. In this case, "ro" (the default).
///         assert_eq!(shot.len(), 2);
///         // In the case of this particular program, we know ro[0] should equal ro[1]
///         assert_eq!(shot[0], shot[1]);
///     }
///
/// def main():
///     import asyncio
///     asyncio.run(run())
/// ```
///
/// # A Note on Lifetimes
///
/// This structure utilizes multiple lifetimes for the sake of runtime efficiency.
/// You should be able to largely ignore these, just keep in mind that any borrowed data passed to
/// the methods most likely needs to live as long as this struct. Check individual methods for
/// specifics. If only using `'static` strings then everything should just work.
#[derive(Clone)]
#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(module = "qcs_sdk", name = "Executable", frozen)]
pub(crate) struct PyExecutable(Arc<Mutex<Executable<'static, 'static>>>);

#[derive(Clone)]
#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(module = "qcs_sdk", name = "JobHandle", frozen)]
pub(crate) struct PyJobHandle(JobHandle<'static>);

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[pymethods]
impl PyJobHandle {
    #[getter]
    pub fn job_id(&self) -> JobId {
        self.0.job_id()
    }

    #[getter]
    pub fn readout_map(&self) -> &HashMap<String, String> {
        self.0.readout_map()
    }
}

/// Invoke a `PyExecutable`'s inner `Executable::method` with given arguments,
/// then mapped to `Future<Output = Result<ExecutionData, ExecutionError>>`.
macro_rules! py_executable_data {
    ($self: ident, $method: ident $(, $arg: expr)* $(,)?) => {{
        let arc = $self.0.clone();
        async move {
            arc.lock()
                .await
                .$method($($arg),*)
                .await
                .map(ExecutionData::from)
                .map_err(Into::into)
        }.with_current_context()
    }};
}

/// Invoke a `PyExecutable`'s inner `Executable::method` with given arguments,
/// then mapped to `Future<Output = Result<PyJobHandle, ExecutionError>>`
macro_rules! py_job_handle {
    ($self: ident, $method: ident $(, $arg: expr)* $(,)?) => {{
        let arc = $self.0.clone();
        async move {
            arc.lock()
                .await
                .$method($($arg),*)
                .await
                .map(JobHandle::from)
                .map(PyJobHandle)
                .map_err(Into::into)
        }
        .with_current_context()
    }};
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyExecutable {
    #[new]
    #[pyo3(signature = (
        quil,
        registers = Vec::new(),
        parameters = Vec::new(),
        shots = None,
        quilc_client = None,
        compiler_options = None,
    ))]
    pub(crate) fn __new__(
        quil: String,
        registers: Vec<String>,
        parameters: Vec<ExeParameter>,
        shots: Option<NonZeroU16>,
        quilc_client: Option<PyQuilcClient>,
        compiler_options: Option<CompilerOpts>,
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
            exe = exe.with_shots(shots.0);
        }

        if let Some(options) = compiler_options {
            exe = exe.compiler_options(options);
        }

        Self(Arc::new(Mutex::new(exe)))
    }
}

#[pyo3_opentelemetry::pypropagate(on_context_extraction_failure = "ignore")]
#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyExecutable {
    #[instrument(skip_all)]
    pub fn execute_on_qvm<'py>(
        &self,
        py: Python<'py>,
        client: PyQvmClient,
    ) -> PyResult<ExecutionData> {
        py_sync!(py, py_executable_data!(self, execute_on_qvm, &client))
    }

    #[instrument(skip_all)]
    pub fn execute_on_qvm_async<'py>(
        &self,
        py: Python<'py>,
        client: PyQvmClient,
    ) -> PyResult<Bound<'py, PyAny>> {
        pyo3_async_runtimes::tokio::future_into_py(
            py,
            py_executable_data!(self, execute_on_qvm, &client),
        )
    }

    #[pyo3(signature = (quantum_processor_id, endpoint_id = None, translation_options = None, execution_options = None))]
    pub fn execute_on_qpu(
        &self,
        py: Python<'_>,
        quantum_processor_id: String,
        endpoint_id: Option<String>,
        translation_options: Option<TranslationOptions>,
        execution_options: Option<ExecutionOptions>,
    ) -> PyResult<ExecutionData> {
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
                    &execution_options.unwrap_or_default(),
                )
            ),
        }
    }

    #[pyo3(signature = (quantum_processor_id, endpoint_id = None, translation_options = None, execution_options = None))]
    pub fn execute_on_qpu_async<'py>(
        &self,
        py: Python<'py>,
        quantum_processor_id: String,
        endpoint_id: Option<String>,
        translation_options: Option<TranslationOptions>,
        execution_options: Option<ExecutionOptions>,
    ) -> PyResult<Bound<'py, PyAny>> {
        match endpoint_id {
            Some(endpoint_id) => pyo3_async_runtimes::tokio::future_into_py(
                py,
                py_executable_data!(
                    self,
                    execute_on_qpu_with_endpoint,
                    quantum_processor_id,
                    endpoint_id,
                    translation_options,
                ),
            ),
            None => pyo3_async_runtimes::tokio::future_into_py(
                py,
                py_executable_data!(
                    self,
                    execute_on_qpu,
                    quantum_processor_id,
                    translation_options,
                    &execution_options.unwrap_or_default(),
                ),
            ),
        }
    }

    #[pyo3(signature = (quantum_processor_id, endpoint_id = None, translation_options = None, execution_options = None))]
    pub fn submit_to_qpu(
        &self,
        py: Python<'_>,
        quantum_processor_id: String,
        endpoint_id: Option<String>,
        translation_options: Option<TranslationOptions>,
        execution_options: Option<ExecutionOptions>,
    ) -> PyResult<PyJobHandle> {
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
                    &execution_options.unwrap_or_default(),
                )
            ),
        }
    }

    #[pyo3(signature = (quantum_processor_id, endpoint_id = None, translation_options = None, execution_options = None))]
    pub fn submit_to_qpu_async<'py>(
        &self,
        py: Python<'py>,
        quantum_processor_id: String,
        endpoint_id: Option<String>,
        translation_options: Option<TranslationOptions>,
        execution_options: Option<ExecutionOptions>,
    ) -> PyResult<Bound<'py, PyAny>> {
        match endpoint_id {
            Some(endpoint_id) => pyo3_async_runtimes::tokio::future_into_py(
                py,
                py_job_handle!(
                    self,
                    submit_to_qpu_with_endpoint,
                    quantum_processor_id,
                    endpoint_id,
                    translation_options,
                ),
            ),
            None => pyo3_async_runtimes::tokio::future_into_py(
                py,
                py_job_handle!(
                    self,
                    submit_to_qpu,
                    quantum_processor_id,
                    translation_options,
                    &execution_options.unwrap_or_default(),
                ),
            ),
        }
    }

    pub fn retrieve_results(
        &self,
        py: Python<'_>,
        job_handle: PyJobHandle,
    ) -> PyResult<ExecutionData> {
        py_sync!(
            py,
            py_executable_data!(self, retrieve_results, job_handle.0)
        )
    }

    pub fn retrieve_results_async<'py>(
        &self,
        py: Python<'py>,
        job_handle: PyJobHandle,
    ) -> PyResult<Bound<'py, PyAny>> {
        pyo3_async_runtimes::tokio::future_into_py(
            py,
            py_executable_data!(self, retrieve_results, job_handle.0),
        )
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(module = "qcs_sdk", get_all, set_all)]
pub struct ExeParameter {
    pub name: String,
    pub index: usize,
    pub value: f64,
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl ExeParameter {
    #[new]
    fn new(name: String, index: usize, value: f64) -> Self {
        Self { name, index, value }
    }
}
