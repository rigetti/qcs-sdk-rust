use rigetti_pyo3::create_init_submodule;

pub mod quilc;

create_init_submodule! {
    submodules: [
        "quilc": quilc::init_submodule
    ],
}
