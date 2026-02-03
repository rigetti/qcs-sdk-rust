use rigetti_pyo3::create_init_submodule;

// #[pyo3(name = "experimental", module = "qcs_sdk.qpu", submodule)]
create_init_submodule! {
    submodules: [ "random": random::init_submodule ],
}

mod random {
    use rigetti_pyo3::create_init_submodule;

    use crate::{
        python::errors,
        qpu::experimental::random::{
            choose_random_real_sub_region_indices, lfsr_v1_next, ChooseRandomRealSubRegions,
            PrngSeedValue
        },
    };

    // #[pyo3(name = "random", module = "qcs_sdk.qpu.experimental", submodule)]
    create_init_submodule! {
        classes: [ ChooseRandomRealSubRegions, PrngSeedValue ],
        errors: [ errors::RandomError ],
        funcs: [ choose_random_real_sub_region_indices, lfsr_v1_next ],
    }
}

