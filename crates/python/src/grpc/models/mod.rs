use rigetti_pyo3::create_init_submodule;

pub mod controller;
pub mod translation;

create_init_submodule! {
    submodules: [
        "translation": translation::init_submodule
    ],
}
