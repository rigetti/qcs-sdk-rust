use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use qcs::qpu::experimental::random::{
    choose_random_real_sub_region_indices, lfsr_v1_next, ChooseRandomRealSubRegions, PrngSeedValue,
};

use quil_rs::quil::Quil;
use rigetti_pyo3::pyo3::exceptions::PyValueError;
use rigetti_pyo3::{create_init_submodule, py_wrap_error, wrap_error, ToPythonError};

create_init_submodule! {
    classes: [
        PyPrngSeedValue,
        PyChooseRandomRealSubRegions
    ],
    errors: [
        RandomError
    ],
    funcs: [
        py_lfsr_v1_next,
        py_choose_random_real_sub_region_indices
    ],
}

wrap_error!(RustRandomError(qcs::qpu::experimental::random::Error));
py_wrap_error!(experimental, RustRandomError, RandomError, PyValueError);

#[pyclass(name = "PrngSeedValue")]
#[derive(Clone)]
pub(super) struct PyPrngSeedValue {
    pub(super) inner: PrngSeedValue,
}

#[pymethods]
impl PyPrngSeedValue {
    #[new]
    fn new(seed: u64) -> PyResult<Self> {
        PrngSeedValue::try_new(seed)
            .map(|inner| Self { inner })
            .map_err(RustRandomError::from)
            .map_err(RustRandomError::to_py_err)
    }
}

#[pyclass(name = "ChooseRandomRealSubRegions")]
struct PyChooseRandomRealSubRegions;

#[pymethods]
impl PyChooseRandomRealSubRegions {
    #[classattr]
    const NAME: &'static str = ChooseRandomRealSubRegions::EXTERN_NAME;

    #[classmethod]
    fn build_signature(_cls: &pyo3::types::PyType) -> PyResult<String> {
        ChooseRandomRealSubRegions::build_signature()
            .map_err(RustRandomError::from)
            .map_err(RustRandomError::to_py_err)
            .and_then(|signature| {
                signature.to_quil().map_err(|e| {
                    PyRuntimeError::new_err(format!("failed to write signature as Quil: {e}"))
                        .to_py_err()
                })
            })
    }
}

#[pyfunction(name = "lfsr_v1_next")]
fn py_lfsr_v1_next(seed_value: PyPrngSeedValue) -> PyResult<PyPrngSeedValue> {
    PyPrngSeedValue::new(lfsr_v1_next(seed_value.inner))
}

#[pyfunction(name = "choose_random_real_sub_region_indices")]
fn py_choose_random_real_sub_region_indices(
    seed: PyPrngSeedValue,
    start_index: u32,
    series_length: u32,
    sub_region_count: u8,
) -> Vec<u8> {
    choose_random_real_sub_region_indices(seed.inner, start_index, series_length, sub_region_count)
}
