use numpy::{Complex32, PyArray};
use pyo3::{prelude::*, Bound, IntoPyObjectExt, PyAny, PyResult, Python};

use crate::RegisterData;

#[cfg_attr(feature = "stubs", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl RegisterData {
    #[new]
    fn __new__(values: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(values) = values.extract::<Vec<Vec<i8>>>() {
            Ok(Self::I8(values))
        } else if let Ok(values) = values.extract::<Vec<Vec<f64>>>() {
            Ok(Self::F64(values))
        } else if let Ok(values) = values.extract::<Vec<Vec<i16>>>() {
            Ok(Self::I16(values))
        } else if let Ok(values) = values.extract::<Vec<Vec<Complex32>>>() {
            Ok(Self::Complex32(values))
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                "expected a list of lists of integers, reals, or complex numbers",
            ))
        }
    }

    /// Returns the values as a 2D numpy ndarray.
    pub fn as_ndarray<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        match self {
            RegisterData::I8(matrix) => {
                PyArray::from_vec2(py, matrix.as_slice())?.into_bound_py_any(py)
            }
            RegisterData::F64(matrix) => {
                PyArray::from_vec2(py, matrix.as_slice())?.into_bound_py_any(py)
            }
            RegisterData::I16(matrix) => {
                PyArray::from_vec2(py, matrix.as_slice())?.into_bound_py_any(py)
            }
            RegisterData::Complex32(matrix) => {
                PyArray::from_vec2(py, matrix.as_slice())?.into_bound_py_any(py)
            }
        }
    }
}
