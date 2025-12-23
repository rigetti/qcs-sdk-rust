use pyo3::{prelude::*, types::PyType};
use rigetti_pyo3::create_init_submodule;

#[cfg(feature = "stubs")]
use pyo3_stub_gen::derive::gen_stub_pymethods;

use quil_rs::quil::Quil;

use crate::{
    python::errors,
    qpu::experimental::random::{
        choose_random_real_sub_region_indices, lfsr_v1_next, ChooseRandomRealSubRegions,
        PrngSeedValue, RandomResult,
    },
};

// #[pyo3(name = "experimental", module = "qcs_sdk.qpu", submodule)]
create_init_submodule! {
    submodules: [ "random": random::init_submodule ],
}

mod random {
    use super::*;

    // #[pyo3(name = "random", module = "qcs_sdk.qpu.experimental", submodule)]
    create_init_submodule! {
        classes: [ ChooseRandomRealSubRegions, PrngSeedValue ],
        errors: [ errors::RandomError ],
        funcs: [ choose_random_real_sub_region_indices, lfsr_v1_next ],
    }
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl ChooseRandomRealSubRegions {
    #[classattr]
    const NAME: &'static str = ChooseRandomRealSubRegions::EXTERN_NAME;

    #[classmethod]
    #[pyo3(name = "build_signature")]
    fn py_build_signature(_cls: &Bound<'_, PyType>) -> RandomResult<String> {
        ChooseRandomRealSubRegions::build_signature().and_then(|signature| Ok(signature.to_quil()?))
    }
}
