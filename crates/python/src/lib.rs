use pyo3::prelude::*;
use rigetti_pyo3::create_init_submodule;

pub mod api;
pub mod executable;
pub mod execution_data;
pub mod grpc;
pub mod qpu;
pub mod register_data;

use executable::QcsExecutionError;

// pub use executable::{Error, Executable, ExecuteResultQPU, ExecuteResultQVM, JobHandle, Service};
// pub use execution_data::{Qpu, Qvm, ReadoutMap};
// pub use register_data::RegisterData;

create_init_submodule! {
    classes: [
        execution_data::PyQpu,
        execution_data::PyQvm,
        execution_data::PyReadoutMap,
        executable::PyExecutable,
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
        api::get_quilc_version
    ],
    submodules: [
        "api": api::init_submodule,
        "qpu": qpu::init_submodule
    ],
}

#[pymodule]
fn qcs_sdk(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    init_submodule("qcs_sdk", py, m)
}
