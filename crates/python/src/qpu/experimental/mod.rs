use rigetti_pyo3::create_init_submodule;

mod random;

create_init_submodule! {
    submodules: [
        "random": random::init_submodule
    ],
}
