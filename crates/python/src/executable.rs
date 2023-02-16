use std::sync::Arc;

use pyo3::{pyclass, FromPyObject};
use qcs::{Error, Executable, ExecutionData, JobHandle, Service};
use rigetti_pyo3::{
    impl_as_mut_for_wrapper, py_wrap_error, py_wrap_simple_enum, py_wrap_type,
    pyo3::{exceptions::PyRuntimeError, pymethods, types::PyDict, Py, PyResult, Python},
    wrap_error, PyWrapper, ToPython, ToPythonError,
};
use tokio::sync::Mutex;

use crate::{execution_data::PyExecutionData, qpu::quilc::PyCompilerOpts};

wrap_error!(ExecutionError(Error));

py_wrap_error!(
    executable,
    ExecutionError,
    QcsExecutionError,
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

#[pymethods]
impl PyExecutable {
    #[new]
    #[args(
        "/",
        registers = "Vec::new()",
        parameters = "Vec::new()",
        shots = "None",
        compile_with_quilc = "None",
        compiler_options = "None"
    )]
    pub fn new(
        quil: String,
        registers: Vec<String>,
        parameters: Vec<PyParameter>,
        shots: Option<u16>,
        compile_with_quilc: Option<bool>,
        compiler_options: Option<PyCompilerOpts>,
    ) -> Self {
        let mut exe = Executable::from_quil(quil);

        for reg in registers {
            exe = exe.read_from(reg);
        }

        for param in parameters {
            exe.with_parameter(param.name, param.index, param.value);
        }

        if let Some(shots) = shots {
            exe = exe.with_shots(shots);
        }

        if let Some(compile_with_quilc) = compile_with_quilc {
            exe = exe.compile_with_quilc(compile_with_quilc);
        }

        if let Some(options) = compiler_options {
            exe = exe.compiler_options(options.into_inner());
        }

        Self::from(Arc::new(Mutex::new(exe)))
    }

    pub fn execute_on_qvm(&self) -> PyResult<PyExecutionData> {
        let arc = self.as_inner().clone();
        crate::utils::py_sync!(async move {
            arc.lock()
                .await
                .execute_on_qvm()
                .await
                .map(ExecutionData::from)
                .map(PyExecutionData::from)
                .map_err(ExecutionError::from)
                .map_err(ExecutionError::to_py_err)
        })
    }

    pub fn execute_on_qpu(&self, quantum_processor_id: String) -> PyResult<PyExecutionData> {
        let arc = self.as_inner().clone();
        crate::utils::py_sync!(async move {
            arc.lock()
                .await
                .execute_on_qpu(quantum_processor_id)
                .await
                .map(ExecutionData::from)
                .map(PyExecutionData::from)
                .map_err(ExecutionError::from)
                .map_err(ExecutionError::to_py_err)
        })
    }

    pub fn retrieve_results(&mut self, job_handle: PyJobHandle) -> PyResult<PyExecutionData> {
        let arc = self.as_inner().clone();
        crate::utils::py_sync!(async move {
            arc.lock()
                .await
                .retrieve_results(job_handle.into_inner())
                .await
                .map(ExecutionData::from)
                .map(PyExecutionData::from)
                .map_err(ExecutionError::from)
                .map_err(ExecutionError::to_py_err)
        })
    }
}

py_wrap_simple_enum! {
    PyService(Service) as "Service" {
        Quilc,
        Qvm,
        Qcs,
        Qpu
    }
}

py_wrap_type! {
    PyJobHandle(JobHandle<'static>);
}

#[pymethods]
impl PyJobHandle {
    #[getter]
    pub fn job_id(&self) -> &str {
        self.as_inner().job_id()
    }

    #[getter]
    pub fn readout_map(&self, py: Python) -> PyResult<Py<PyDict>> {
        self.as_inner().readout_map().to_python(py)
    }
}
