use rigetti_pyo3::create_init_submodule;

pub mod models;

create_init_submodule! {
    submodules: [
        "models": models::init_submodule
    ],
}