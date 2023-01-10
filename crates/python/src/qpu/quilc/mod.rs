use pyo3::{exceptions::PyRuntimeError, pymethods};
use qcs::qpu::quilc::{CompilerOpts, TargetDevice, DEFAULT_COMPILER_TIMEOUT};
use rigetti_pyo3::{
    create_init_submodule, py_wrap_error, py_wrap_struct, py_wrap_type, wrap_error,
};

create_init_submodule! {
    classes: [
        PyCompilerOpts,
        PyTargetDevice
    ],
    consts: [DEFAULT_COMPILER_TIMEOUT],
    errors: [QuilcError],
}

py_wrap_type! {
    #[derive(Default)]
    PyCompilerOpts(CompilerOpts) as "CompilerOpts";
}

#[pymethods]
impl PyCompilerOpts {
    #[new]
    #[args("/", timeout = "DEFAULT_COMPILER_TIMEOUT")]
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

py_wrap_struct! {
    PyTargetDevice(TargetDevice) as "TargetDevice" {}
}
