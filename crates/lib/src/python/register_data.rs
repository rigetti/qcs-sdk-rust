use numpy::PyArray;
use pyo3::{Bound, IntoPyObjectExt, PyAny, PyResult, Python};

use crate::RegisterData;

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pyo3::pymethods]
impl RegisterData {
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
        .map_err(Into::into)
    }
}
