use pyo3::exceptions::PyRuntimeError;
use rigetti_pyo3::{create_init_submodule, py_wrap_error, wrap_error};

pub mod client;
pub mod isa;
pub mod quilc;
pub mod result_data;

create_init_submodule! {
    classes: [result_data::PyReadoutValues, result_data::PyQpuResultData],
    errors: [QcsIsaError],
    submodules: [
        "client": client::init_submodule,
        "isa": isa::init_submodule,
        "quilc": quilc::init_submodule,
        "result_data": result_data::init_submodule
    ],
}

wrap_error! {
    IsaError(qcs::qpu::IsaError);
}

py_wrap_error!(qpu, IsaError, QcsIsaError, PyRuntimeError);
