use pyo3::{prelude::*, types::PyType, wrap_pymodule};

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

#[pymodule]
#[pyo3(name = "experimental", module = "qcs_sdk.qpu", submodule)]
pub(crate) fn init_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(init_submodule_random))?;
    init_submodule_random(m)?;

    Ok(())
}

#[pymodule]
#[pyo3(name = "random", module = "qcs_sdk.qpu.experimental", submodule)]
pub(crate) fn init_submodule_random(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();

    m.add("RandomError", py.get_type::<errors::RandomError>())?;

    m.add_class::<ChooseRandomRealSubRegions>()?;
    m.add_class::<PrngSeedValue>()?;

    m.add_function(wrap_pyfunction!(choose_random_real_sub_region_indices, m)?)?;
    m.add_function(wrap_pyfunction!(lfsr_v1_next, m)?)?;

    Ok(())
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
