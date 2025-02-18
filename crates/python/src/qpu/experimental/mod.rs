use rigetti_pyo3::create_init_submodule;

mod random;
mod randomized_measurements;

create_init_submodule! {
    submodules: [
        "random": random::init_submodule,
        "randomized_measurements": randomized_measurements::init_submodule
    ],
}
