use std::sync::Arc;

use qcs::{Error, Executable, JobHandle, Qpu, Qvm, Service};
use rigetti_pyo3::{
    impl_as_mut_for_wrapper, impl_repr, py_wrap_data_struct, py_wrap_error, py_wrap_simple_enum,
    py_wrap_type,
    pyo3::{
        exceptions::PyRuntimeError,
        pymethods,
        types::{PyDict, PyFloat, PyInt, PyString},
        Py, PyAny, PyResult, Python,
    },
    wrap_error, PyWrapper, ToPython, ToPythonError,
};
use tokio::sync::Mutex;

use crate::qpu::quilc::PyCompilerOpts;

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

#[derive(Clone, Debug)]
pub struct ExeParameter {
    pub name: String,
    pub index: usize,
    pub value: f64,
}

py_wrap_data_struct! {
    PyParameter(ExeParameter) as "ExeParameter" {
        name: String => Py<PyString>,
        index: usize => Py<PyInt>,
        value: f64 => Py<PyFloat>
    }
}

impl_repr!(PyParameter);

#[pymethods]
impl PyParameter {
    #[new]
    pub fn new(name: String, index: usize, value: f64) -> Self {
        Self(ExeParameter { name, index, value })
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
            let ExeParameter { name, index, value } = param.into();
            exe.with_parameter(name, index, value);
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

    pub fn execute_on_qvm<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let arc = self.as_inner().clone();
        pyo3_asyncio::tokio::local_future_into_py(py, async move {
            arc.lock()
                .await
                .execute_on_qvm()
                .await
                .map(Qvm::from)
                .map(|qvm| Python::with_gil(|py| qvm.to_python(py)))
                .map_err(ExecutionError::from)
                .map_err(ExecutionError::to_py_err)?
        })
    }

    pub fn execute_on_qpu<'py>(
        &self,
        py: Python<'py>,
        quantum_processor_id: String,
    ) -> PyResult<&'py PyAny> {
        let arc = self.as_inner().clone();
        pyo3_asyncio::tokio::local_future_into_py(py, async move {
            arc.lock()
                .await
                .execute_on_qpu(quantum_processor_id)
                .await
                .map(Qpu::from)
                .map(|qpu| Python::with_gil(|py| qpu.to_python(py)))
                .map_err(ExecutionError::from)
                .map_err(ExecutionError::to_py_err)?
        })
    }

    pub fn retrieve_results<'py>(
        &mut self,
        py: Python<'py>,
        job_handle: PyJobHandle,
    ) -> PyResult<&'py PyAny> {
        let arc = self.as_inner().clone();
        pyo3_asyncio::tokio::local_future_into_py(py, async move {
            arc.lock()
                .await
                .retrieve_results(job_handle.into_inner())
                .await
                .map(Qpu::from)
                .map(|qpu| Python::with_gil(|py| qpu.to_python(py)))
                .map_err(ExecutionError::from)
                .map_err(ExecutionError::to_py_err)?
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
