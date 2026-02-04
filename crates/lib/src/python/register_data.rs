use numpy::{Complex32, PyArray};
use pyo3::{prelude::*, Bound, IntoPyObjectExt, PyAny, PyResult, Python};

use crate::RegisterData;

#[derive(FromPyObject, IntoPyObject)]
enum PyRegisterData {
    I8(Vec<Vec<i8>>),
    F64(Vec<Vec<f64>>),
    I16(Vec<Vec<i16>>),
    Complex32(Vec<Vec<Complex32>>),
}

#[cfg(feature = "stubs")]
pyo3_stub_gen::impl_stub_type!(PyRegisterData = Vec<Vec<i8>> | Vec<Vec<f64>> | Vec<Vec<i16>> | Vec<Vec<Complex32>>);

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl RegisterData {
    #[new]
    fn __new__(inner: PyRegisterData) -> Self {
        match inner {
            PyRegisterData::I8(matrix) => RegisterData::I8(matrix),
            PyRegisterData::F64(matrix) => RegisterData::F64(matrix),
            PyRegisterData::I16(matrix) => RegisterData::I16(matrix),
            PyRegisterData::Complex32(matrix) => RegisterData::Complex32(matrix),
        }
    }

    fn __getnewargs__(&self) -> (PyRegisterData,) {
        (self.inner(),)
    }

    /// Return the inner values as a 2D Numpy ``ndarray``.
    #[gen_stub(override_return_type(type_repr = "numpy.ndarray"))]
    fn as_ndarray<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
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

    fn inner(&self) -> PyRegisterData {
        match self {
            RegisterData::I8(matrix) => PyRegisterData::I8(matrix.clone()),
            RegisterData::F64(matrix) => PyRegisterData::F64(matrix.clone()),
            RegisterData::I16(matrix) => PyRegisterData::I16(matrix.clone()),
            RegisterData::Complex32(matrix) => PyRegisterData::Complex32(matrix.clone()),
        }
    }
}
