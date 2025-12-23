//! Python bindings for the qcs-sdk crate.
//!
//! While this package can be used directly,
//! [PyQuil](https://pypi.org/project/pyquil/) offers more functionality
//! and a higher-level interface for building and executing Quil programs.

use std::sync::OnceLock;

use pyo3::prelude::*;
use rigetti_pyo3::{create_init_submodule, py_sync};

#[cfg(feature = "stubs")]
use pyo3_stub_gen::{define_stub_info_gatherer, derive::gen_stub_pyfunction};

use crate::{
    client::Qcs,
    compiler,
    python::{
        executable::{ExeParameter, PyExecutable, PyJobHandle},
        execution_data::PyRegisterMatrix,
    },
    qpu, qvm, ExecutionData, RegisterData, RegisterMap, ResultData, Service,
};

pub(crate) mod client;
pub(crate) mod errors;
pub(crate) mod executable;
pub(crate) mod execution_data;
pub(crate) mod nonzero;
pub(crate) mod register_data;

pub(crate) use nonzero::NonZeroU16;

static PY_RESET_LOGGING_HANDLE: OnceLock<pyo3_log::ResetHandle> = OnceLock::new();

create_init_submodule! {
    classes: [
        ExeParameter,
        ExecutionData,
        PyExecutable,
        PyJobHandle,
        PyRegisterMatrix,
        Qcs,
        RegisterMap,
        Service
    ],
    complex_enums: [
        RegisterData,
        ResultData
    ],
    errors: [ errors::QcsSdkError, errors::ExecutionError, errors::RegisterMatrixConversionError ],
    funcs: [ reset_logging, gather_diagnostics ],
    submodules: [
        "client": client::init_submodule,
        "compiler": compiler::python::init_submodule,
        "qpu": qpu::python::init_submodule,
        "qvm": qvm::python::init_submodule
    ],
}

#[pymodule]
#[pyo3(name = "_qcs_sdk")]
fn init_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    match pyo3_log::try_init() {
        Ok(reset_handle) => {
            // Ignore the error if the handle is already set.
            drop(PY_RESET_LOGGING_HANDLE.set(reset_handle));
        }
        Err(e) => eprintln!("Failed to initialize the qcs_sdk logger: {e}"),
    }

    let py = m.py();
    init_submodule("qcs_sdk", py, m)?;

    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    pyo3_tracing_subscriber::add_submodule("qcs_sdk", "_tracing_subscriber", py, m)?;

    Ok(())
}

#[cfg(feature = "stubs")]
define_stub_info_gatherer!(stub_info);

#[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk"))]
#[pyfunction]
fn reset_logging() {
    if let Some(handle) = PY_RESET_LOGGING_HANDLE.get() {
        handle.reset();
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk"))]
#[pyfunction]
#[pyo3(name = "_gather_diagnostics")]
fn gather_diagnostics(py: Python<'_>) -> PyResult<String> {
    py_sync!(py, async { Ok(crate::diagnostics::get_report().await) })
}
