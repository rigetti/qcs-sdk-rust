use pyo3::prelude::*;
use rigetti_pyo3::create_init_submodule;

use executable::QCSExecutionError;

pub mod api;
pub mod executable;
pub mod execution_data;
pub mod grpc;
pub mod qpu;
pub mod qvm;
pub mod register_data;

pub(crate) mod py_sync;

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
        QCSExecutionError,
        execution_data::PyRegisterMatrixConversionError
    ],
    funcs: [
        api::rewrite_arithmetic,
        api::py_translate,
        api::py_translate_async,
        api::py_submit,
        api::py_submit_async,
        api::py_retrieve_results,
        api::py_retrieve_results_async,
        api::build_patch_values,
        api::py_list_quantum_processors,
        api::py_list_quantum_processors_async,
        qpu::isa::py_get_instruction_set_architecture,
        qpu::isa::py_get_instruction_set_architecture_async
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
