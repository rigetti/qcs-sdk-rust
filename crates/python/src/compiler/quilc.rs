use pyo3::exceptions::PyValueError;
use pyo3::{exceptions::PyRuntimeError, pyfunction, pymethods, PyResult};
use qcs::compiler::quilc::{CompilerOpts, TargetDevice, DEFAULT_COMPILER_TIMEOUT};
use qcs_api_client_openapi::models::InstructionSetArchitecture;
use rigetti_pyo3::{
    create_init_submodule, py_wrap_error, py_wrap_struct, py_wrap_type, wrap_error, ToPythonError,
};

use crate::py_sync::py_function_sync_async;
use crate::qpu::client::PyQcsClient;
use crate::qpu::isa::PyInstructionSetArchitecture;

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
    #[args("/", timeout = "DEFAULT_COMPILER_TIMEOUT", protoquil = "None")]
    pub fn new(timeout: Option<f64>, protoquil: Option<bool>) -> Self {
        let opts = CompilerOpts::new()
            .with_timeout(timeout)
            .with_protoquil(protoquil);
        Self(opts)
    }

    #[staticmethod]
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Self {
        <Self as Default>::default()
    }
}

wrap_error!(RustQuilcError(qcs::compiler::quilc::Error));
py_wrap_error!(quilc, RustQuilcError, QuilcError, PyRuntimeError);

py_wrap_struct! {
    PyTargetDevice(TargetDevice) as "TargetDevice" {}
}

#[pymethods]
impl PyTargetDevice {
    #[staticmethod]
    pub fn from_isa(isa: PyInstructionSetArchitecture) -> PyResult<Self> {
        let isa: InstructionSetArchitecture = isa.into();
        let target: TargetDevice = isa
            .try_into()
            .map_err(RustQuilcError::from)
            .map_err(RustQuilcError::to_py_err)?;

        Ok(Self(target))
    }

    #[staticmethod]
    pub fn from_json(value: String) -> PyResult<Self> {
        let target: TargetDevice = serde_json::from_str(&value)
            .map_err(|err| err.to_string())
            .map_err(PyValueError::new_err)?;

        Ok(Self(target))
    }
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
        qcs::compiler::quilc::compile_program(&quil, target.into(), &client, options.into())
            .map_err(RustQuilcError::from)
            .map_err(RustQuilcError::to_py_err)
            .map(|p| p.to_string())
    }
}

py_function_sync_async! {
    #[pyfunction(client = "None")]
    async fn get_version_info(
        client: Option<PyQcsClient>,
    ) -> PyResult<String> {
        let client = PyQcsClient::get_or_create_client(client).await?;
        qcs::compiler::quilc::get_version_info(&client)
            .map_err(RustQuilcError::from)
            .map_err(RustQuilcError::to_py_err)
    }
}
