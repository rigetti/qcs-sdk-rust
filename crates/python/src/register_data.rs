use numpy::PyArray;
use pyo3::{
    pymethods,
    types::{PyComplex, PyFloat, PyInt},
    Py, PyErr, PyObject, PyResult, Python, ToPyObject,
};
use qcs::RegisterData;
use rigetti_pyo3::{py_wrap_union_enum, PyWrapper};

py_wrap_union_enum! {
    PyRegisterData(RegisterData) as "RegisterData" {
        i8: I8 => Vec<Vec<Py<PyInt>>>,
        f64: F64 => Vec<Vec<Py<PyFloat>>>,
        i16: I16 => Vec<Vec<Py<PyInt>>>,
        complex32: Complex32 => Vec<Vec<Py<PyComplex>>>
    }
}

#[pymethods]
impl PyRegisterData {
    pub fn as_ndarray(&self, py: Python<'_>) -> PyResult<PyObject> {
        match self.as_inner() {
            RegisterData::I8(matrix) => {
                PyArray::from_vec2(py, matrix.as_slice()).map(|arr| arr.to_object(py))
            }
            RegisterData::F64(matrix) => {
                PyArray::from_vec2(py, matrix.as_slice()).map(|arr| arr.to_object(py))
            }
            RegisterData::I16(matrix) => {
                PyArray::from_vec2(py, matrix.as_slice()).map(|arr| arr.to_object(py))
            }
            RegisterData::Complex32(matrix) => {
                PyArray::from_vec2(py, matrix.as_slice()).map(|arr| arr.to_object(py))
            }
        }
        .map_err(PyErr::from)
    }
}
