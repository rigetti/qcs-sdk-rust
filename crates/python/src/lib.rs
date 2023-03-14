use pyo3::prelude::*;
use quil;
use rigetti_pyo3::create_init_submodule;

use executable::ExecutionError;
use execution_data::RegisterMatrixConversionError;

pub mod compiler;
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
        ExecutionError,
        RegisterMatrixConversionError
    ],
    submodules: [
        "compiler": compiler::init_submodule,
        "qpu": qpu::init_submodule,
        "qvm": qvm::init_submodule,
        "quil": quil::init_quil_submodule
    ],
}

#[pymodule]
fn qcs_sdk(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    init_submodule("qcs_sdk", py, m)
}
