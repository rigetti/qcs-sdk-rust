//! Python bindings for the experimental QPU module.

use quil_rs::quil::{Quil, ToQuilError};
use rigetti_pyo3::create_init_submodule;

use crate::qpu::experimental::random::ChooseRandomRealSubRegions;

create_init_submodule! {
    submodules: [ "random": random::init_submodule ],
}

mod random {
    use rigetti_pyo3::create_init_submodule;

    use crate::{
        python::errors,
        qpu::experimental::random::{
            choose_random_real_sub_region_indices, lfsr_v1_next, ChooseRandomRealSubRegions,
            PrngSeedValue,
        },
    };

    create_init_submodule! {
        classes: [ ChooseRandomRealSubRegions, PrngSeedValue ],
        errors: [ errors::RandomError ],
        funcs: [ choose_random_real_sub_region_indices, lfsr_v1_next ],
    }
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pyo3::pymethods]
impl ChooseRandomRealSubRegions {
    /// Build the signature for the `PRAGMA EXTERN choose_random_real_sub_regions` instruction.
    ///
    /// The signature expressed in Quil is as follows:
    ///
    /// ```text
    /// "(destination : mut REAL[], source : REAL[], sub_region_size : INTEGER, seed : mut INTEGER)"
    /// ```
    #[staticmethod]
    #[pyo3(name = "build_signature")]
    fn py_build_signature() -> Result<String, ToQuilError> {
        ChooseRandomRealSubRegions::build_signature().to_quil()
    }
}
