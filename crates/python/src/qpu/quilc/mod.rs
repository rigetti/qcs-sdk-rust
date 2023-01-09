use pyo3::{exceptions::PyRuntimeError, pymethods};
use qcs::qpu::quilc::CompilerOpts;
use rigetti_pyo3::{py_wrap_error, py_wrap_type, wrap_error};

py_wrap_type! {
    #[derive(Default)]
    PyCompilerOpts(CompilerOpts) as "CompilerOpts";
}

#[pymethods]
impl PyCompilerOpts {
    #[new]
    pub fn new(timeout: Option<u8>) -> Self {
        Self::from(CompilerOpts::new().with_timeout(timeout))
    }

    #[staticmethod]
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Self {
        <Self as Default>::default()
    }
}

wrap_error!(Error(qcs::qpu::quilc::Error));

py_wrap_error!(quilc, Error, QuilcError, PyRuntimeError);

// TODO: TargetDevice
