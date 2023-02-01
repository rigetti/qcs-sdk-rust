use pyo3::prelude::*;
use rigetti_pyo3::create_init_submodule;

use executable::QcsExecutionError;

pub mod api;
pub mod executable;
pub mod execution_data;
pub mod grpc;
pub mod qpu;
pub mod qvm;
pub mod register_data;

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
        qpu::client::PyQcsClient
    ],
    errors: [
        QcsExecutionError
    ],
    funcs: [
        api::compile,
        api::rewrite_arithmetic,
        api::translate,
        api::submit,
        api::retrieve_results,
        api::build_patch_values,
        api::get_quilc_version,
        api::py_list_quantum_processors
    ],
    submodules: [
        "api": api::init_submodule,
        "qpu": qpu::init_submodule,
        "qvm": qvm::init_submodule
    ],
}

#[pymodule]
fn qcs_sdk(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    init_submodule("qcs_sdk", py, m)
}
