//! Python bindings for the qcs-sdk crate.
//!
//! While this package can be used directly,
//! [PyQuil](https://pypi.org/project/pyquil/) offers more functionality
//! and a higher-level interface for building and executing Quil programs.

use std::sync::OnceLock;

use pyo3::prelude::*;
use rigetti_pyo3::{create_init_submodule, py_sync};

#[cfg(feature = "stubs")]
use pyo3_stub_gen::derive::gen_stub_pyfunction;

use crate::{
    compiler, diagnostics,
    python::{
        executable::{ExeParameter, PyExecutable, PyJobHandle},
        execution_data::{
            PyRegisterMatrix, RegisterMapItemsIter, RegisterMapKeysIter, RegisterMapValuesIter,
        },
    },
    qpu, qvm, ExecutionData, RegisterData, RegisterMap, Service,
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
        RegisterMap,
        RegisterMapItemsIter,
        RegisterMapKeysIter,
        RegisterMapValuesIter,
        Service
    ],

    complex_enums: [ PyRegisterMatrix, RegisterData ],

    errors: [
        errors::QcsSdkError,
        errors::ExecutionError,
        errors::RegisterMatrixConversionError
    ],

    funcs: [ reset_logging, gather_diagnostics ],

    submodules: [
        "client": client::init_submodule,
        "compiler": compiler::python::init_submodule,
        "qpu": qpu::python::init_submodule,
        "qvm": qvm::python::init_submodule,
        "diagnostics": diagnostics::python::init_submodule
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
mod stubs {
    use pyo3_stub_gen::{define_stub_info_gatherer, generate::Module, Result, StubInfo};
    use std::path::Path;

    #[derive(pyo3::IntoPyObject)]
    struct Final<T>(T);

    impl<T> pyo3_stub_gen::PyStubType for Final<T> {
        fn type_output() -> pyo3_stub_gen::TypeInfo {
            pyo3_stub_gen::TypeInfo::with_module("typing.Final", "typing".into())
        }
    }

    pyo3_stub_gen::module_variable!(
        "qcs_sdk",
        "__version__",
        Final<&str>,
        Final(env!("CARGO_PKG_VERSION"))
    );

    define_stub_info_gatherer!(internal_stub_info);

    /// Ensures modules exist in the stubs, even if they don't have any members.
    fn ensure_submod(stubs: &mut StubInfo, module: &str) {
        stubs.modules.entry(module.to_string()).or_insert(Module {
            name: module.to_string(),
            ..Default::default()
        });

        if let Some((parent, child)) = module.rsplit_once('.') {
            stubs
                .modules
                .entry(parent.to_string())
                .or_insert(Module {
                    name: parent.to_string(),
                    ..Default::default()
                })
                .submodules
                .insert(child.to_string());

            ensure_submod(stubs, parent);
        }
    }

    /// Gather stub information to generate stub files.
    pub fn stub_info() -> Result<StubInfo> {
        let manifest_dir: &Path = env!("CARGO_MANIFEST_DIR").as_ref();
        let mut stubs = StubInfo::from_pyproject_toml(manifest_dir.join("pyproject.toml"))?;

        // Add otherwise empty modules, as they aren't found automatically.
        let module_leaves = [
            "qcs_sdk.compiler.quilc",
            "qcs_sdk.qpu.experimental.random",
            "qcs_sdk.qvm",
        ];

        for mod_name in module_leaves {
            ensure_submod(&mut stubs, mod_name);
        }

        Ok(stubs)
    }
}

#[cfg(feature = "stubs")]
pub use stubs::stub_info;

#[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk"))]
#[pyfunction]
/// Reset all caches for logging configuration within this library,
/// allowing the most recent Python-side changes to be applied.
///
/// See <https://docs.rs/pyo3-log/latest/pyo3_log/> for more information.
fn reset_logging() {
    if let Some(handle) = PY_RESET_LOGGING_HANDLE.get() {
        handle.reset();
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk"))]
#[pyfunction]
#[pyo3(name = "_gather_diagnostics")]
fn gather_diagnostics(py: Python<'_>) -> PyResult<String> {
    py_sync!(py, async { Ok(diagnostics::get_report().await) })
}
