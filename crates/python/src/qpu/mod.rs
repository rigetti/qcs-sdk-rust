use rigetti_pyo3::create_init_submodule;

pub use result_data::{PyQpuResultData, PyReadoutValues};

pub mod client;
pub mod isa;
pub mod quilc;
mod result_data;

use isa::QCSISAError;

create_init_submodule! {
    classes: [PyQpuResultData, PyReadoutValues],
    errors: [QCSISAError],
    submodules: [
        "client": client::init_submodule,
        "isa": isa::init_submodule,
        "quilc": quilc::init_submodule
    ],
}
