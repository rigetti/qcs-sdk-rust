use std::collections::HashMap;
use std::str::FromStr;

use numpy::{Complex64, PyArray2};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use qcs::qpu::experimental::random::{
    choose_random_real_sub_region_indices, lfsr_v1_next, ChooseRandomRealSubRegions, PrngSeedValue,
};
use qcs::qpu::experimental::randomized_measurements::{RandomizedMeasurements, UnitarySet};
use qcs::qpu::externed_call::ExternedCall;
use quil_rs::instruction::{Measurement, MemoryReference, Qubit};
use quil_rs::quil::Quil;
use rigetti_pyo3::pyo3::exceptions::PyValueError;
use rigetti_pyo3::{
    create_init_submodule, py_wrap_error, py_wrap_type, wrap_error, PyWrapper, ToPythonError,
};

create_init_submodule! {
    classes: [
        PyPrngSeedValue,
        PyRandomizedMeasurement,
        PyRandomizedMeasurements,
        PyUnitarySet,
        PyChooseRandomRealSubRegions
    ],
    errors: [
        RandomError,
        RandomizedMeasurementsError
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
struct PyPrngSeedValue {
    inner: PrngSeedValue,
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
    const NAME: &'static str = ChooseRandomRealSubRegions::NAME;

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

wrap_error!(RustRandomizedMeasurementsError(
    qcs::qpu::experimental::randomized_measurements::Error
));
py_wrap_error!(
    experimental,
    RustRandomizedMeasurementsError,
    RandomizedMeasurementsError,
    PyValueError
);

py_wrap_type! {
    PyUnitarySet(UnitarySet) as "UnitarySet";
}
rigetti_pyo3::impl_repr!(PyUnitarySet);

#[pymethods]
impl PyUnitarySet {
    #[staticmethod]
    fn from_zxzxz(inner: &PyArray2<f64>) -> PyUnitarySet {
        Self(UnitarySet::Zxzxz(inner.to_owned_array()))
    }

    fn to_zxzxz<'a>(&self, py: Python<'a>) -> PyResult<&'a PyArray2<f64>> {
        let UnitarySet::Zxzxz(matrix) = self.as_inner();
        Ok(PyArray2::from_array(py, matrix))
    }

    fn as_zxzxz<'a>(&self, py: Python<'a>) -> Option<&'a PyArray2<f64>> {
        self.to_zxzxz(py).ok()
    }

    fn is_zxzxz(&self) -> bool {
        matches!(self.as_inner(), UnitarySet::Zxzxz(_))
    }
}

#[pyclass(name = "RandomizedMeasurement")]
#[derive(Clone)]
struct PyRandomizedMeasurement {
    inner: Measurement,
}

#[pymethods]
impl PyRandomizedMeasurement {
    #[new]
    fn new(qubit: u64, target: (String, u64)) -> Self {
        Self {
            inner: Measurement {
                qubit: Qubit::Fixed(qubit),
                target: Some(MemoryReference {
                    name: target.0,
                    index: target.1,
                }),
            },
        }
    }
}

#[pyclass(name = "RandomizedMeasurements")]
#[derive(Clone)]
struct PyRandomizedMeasurements {
    inner: RandomizedMeasurements,
}

impl From<PyRandomizedMeasurement> for Measurement {
    fn from(py_measurement: PyRandomizedMeasurement) -> Self {
        py_measurement.inner
    }
}

#[pymethods]
impl PyRandomizedMeasurements {
    #[new]
    #[pyo3(signature = (measurements, unitary_set, leading_delay = 1e-5))]
    fn new(
        measurements: Vec<PyRandomizedMeasurement>,
        unitary_set: PyUnitarySet,
        leading_delay: f64,
    ) -> PyResult<Self> {
        RandomizedMeasurements::try_new(
            measurements.into_iter().map(Measurement::from).collect(),
            unitary_set.as_inner().clone(),
            quil_rs::expression::Expression::Number(Complex64 {
                re: leading_delay,
                im: 0.0,
            }),
        )
        .map(|inner| Self { inner })
        .map_err(RustRandomizedMeasurementsError::from)
        .map_err(RustRandomizedMeasurementsError::to_py_err)
    }

    fn append_to_program(&self, target_program: String) -> PyResult<String> {
        let target_program = quil_rs::Program::from_str(target_program.as_str())
            .map_err(|e| PyValueError::new_err(format!("failed to parse target program: {e}")))?;
        self.inner
            .append_to_program(target_program)
            .map_err(RustRandomizedMeasurementsError::from)
            .map_err(RustRandomizedMeasurementsError::to_py_err)
            .and_then(|program| {
                program.to_quil().map_err(|e| {
                    PyRuntimeError::new_err(format!("failed to write program as Quil: {e}"))
                })
            })
    }

    fn to_parameters(
        &self,
        seed_values: HashMap<u64, PyPrngSeedValue>,
    ) -> PyResult<HashMap<String, Vec<f64>>> {
        self.inner
            .to_parameters(
                &seed_values
                    .into_iter()
                    .map(|(k, v)| (Qubit::Fixed(k), v.inner))
                    .collect(),
            )
            .map(|parameters| {
                parameters
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect()
            })
            .map_err(RustRandomizedMeasurementsError::from)
            .map_err(RustRandomizedMeasurementsError::to_py_err)
    }

    fn get_random_indices(
        &self,
        seed_values: HashMap<u64, PyPrngSeedValue>,
        shot_count: u32,
    ) -> PyResult<HashMap<u64, Vec<u8>>> {
        self.inner
            .get_random_indices(
                &seed_values
                    .into_iter()
                    .map(|(k, v)| (Qubit::Fixed(k), v.inner))
                    .collect(),
                shot_count,
            )
            .into_iter()
            .map(|(qubit, sequence)| match qubit {
                Qubit::Fixed(qubit) => Ok((qubit, sequence)),
                _ => Err(PyRuntimeError::new_err(
                    "The Rust implementation erroneously produced non-fixed qubits",
                )
                .to_py_err()),
            })
            .collect::<Result<HashMap<u64, Vec<u8>>, PyErr>>()
    }
}
