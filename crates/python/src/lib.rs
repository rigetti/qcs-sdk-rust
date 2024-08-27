use std::sync::Mutex;

use pyo3::prelude::*;
use rigetti_pyo3::{create_init_submodule, py_sync};

use executable::ExecutionError;
use execution_data::RegisterMatrixConversionError;

pub mod client;
pub mod compiler;
pub mod executable;
pub mod execution_data;
pub mod grpc;
pub mod qpu;
pub mod qvm;
pub mod register_data;

pub(crate) mod from_py;

create_init_submodule! {
    classes: [
        execution_data::PyExecutionData,
        execution_data::PyResultData,
        execution_data::PyRegisterMap,
        execution_data::PyRegisterMatrix,
        executable::PyExecutable,
        executable::PyParameter,
        executable::PyJobHandle,
        executable::PyService,
        register_data::PyRegisterData,
        client::PyQcsClient
    ],
    errors: [
        ExecutionError,
        RegisterMatrixConversionError
    ],
    funcs: [ reset_logging, gather_diagnostics ],
    submodules: [
        "client": client::init_submodule,
        "compiler": compiler::init_submodule,
        "qpu": qpu::init_submodule,
        "qvm": qvm::init_submodule
    ],
}

static PY_RESET_LOGGING_HANDLE: once_cell::sync::Lazy<Mutex<Option<pyo3_log::ResetHandle>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(None));

#[pymodule]
fn qcs_sdk(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    match pyo3_log::try_init() {
        Ok(reset_handle) => {
            if let Ok(mut handle) = PY_RESET_LOGGING_HANDLE.lock() {
                *handle = Some(reset_handle);
            }
        }
        Err(e) => eprintln!("Failed to initialize the qcs_sdk logger: {e}"),
    }
    init_submodule("qcs_sdk", py, m)?;
    pyo3_tracing_subscriber::add_submodule("qcs_sdk", "_tracing_subscriber", py, m)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))
}

#[pyfunction]
fn reset_logging() {
    if let Ok(handle) = PY_RESET_LOGGING_HANDLE.lock() {
        if let Some(handle) = handle.as_ref() {
            handle.reset();
        }
    }
}

#[pyfunction]
#[pyo3(name = "_gather_diagnostics")]
fn gather_diagnostics(py: Python<'_>) -> PyResult<String> {
    py_sync!(py, async { Ok(qcs::diagnostics::get_report().await) })
}
