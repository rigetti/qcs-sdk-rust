use pyo3::{exceptions::PyRuntimeError, pyfunction, pymethods, PyResult};
use qcs::qpu::quilc::{CompilerOpts, TargetDevice, DEFAULT_COMPILER_TIMEOUT};
use rigetti_pyo3::{
    create_init_submodule, py_wrap_error, py_wrap_struct, py_wrap_type, wrap_error, ToPythonError,
};

use crate::py_sync::py_function_sync_async;

use super::client::PyQcsClient;

create_init_submodule! {
    classes: [
        PyCompilerOpts,
        PyTargetDevice
    ],
    consts: [DEFAULT_COMPILER_TIMEOUT],
    errors: [QuilcError],
    funcs: [
        py_compile_program,
        py_compile_program_async,
        py_get_version_info,
        py_get_version_info_async
    ],
}

py_wrap_type! {
    #[derive(Default)]
    PyCompilerOpts(CompilerOpts) as "CompilerOpts";
}

#[pymethods]
impl PyCompilerOpts {
    #[new]
    #[args("/", timeout = "DEFAULT_COMPILER_TIMEOUT")]
    pub fn new(timeout: Option<f64>) -> Self {
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

py_function_sync_async! {
    #[pyfunction(client = "None", options = "None")]
    async fn compile_program(
        quil: String,
        target: PyTargetDevice,
        client: Option<PyQcsClient>,
        options: Option<PyCompilerOpts>,
    ) -> PyResult<String> {
        let client = PyQcsClient::get_or_create_client(client).await?;
        let options = options.unwrap_or_default();
        qcs::qpu::quilc::compile_program(&quil, target.into(), &client, options.into())
            .map_err(Error::from)
            .map_err(Error::to_py_err)
            .map(|p| p.to_string(true))
    }
}

py_function_sync_async! {
    #[pyfunction(client = "None")]
    async fn get_version_info(
        client: Option<PyQcsClient>,
    ) -> PyResult<String> {
        let client = PyQcsClient::get_or_create_client(client).await?;
        qcs::qpu::quilc::get_version_info(&client)
            .map_err(Error::from)
            .map_err(Error::to_py_err)
    }
}
