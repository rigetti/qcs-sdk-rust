use pyo3::prelude::*;
use rigetti_pyo3::create_init_submodule;

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
        client::PyQcsClient
    ],
    errors: [
        ExecutionError,
        RegisterMatrixConversionError
    ],
    submodules: [
        "client": client::init_submodule,
        "compiler": compiler::init_submodule,
        "qpu": qpu::init_submodule,
        "qvm": qvm::init_submodule
    ],
}

#[pymodule]
fn qcs_sdk(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    pyo3_log::init();
    init_submodule("qcs_sdk", py, m)
}
