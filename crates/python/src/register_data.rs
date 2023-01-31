use pyo3::{
    types::{PyComplex, PyFloat, PyInt},
    Py,
};
use qcs::RegisterData;
use rigetti_pyo3::py_wrap_union_enum;

py_wrap_union_enum! {
    PyRegisterData(RegisterData) as "RegisterData" {
        i8: I8 => Vec<Vec<Py<PyInt>>>,
        f64: F64 => Vec<Vec<Py<PyFloat>>>,
        i16: I16 => Vec<Vec<Py<PyInt>>>,
        complex32: Complex32 => Vec<Vec<Py<PyComplex>>>
    }
}
