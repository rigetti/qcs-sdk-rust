use pyo3::{
    prelude::*,
    types::{PyComplex, PyInt},
};
use qcs_api_client_grpc::models::controller::{
    readout_values::Values, Complex64, Complex64ReadoutValues, IntegerReadoutValues, ReadoutValues,
};
use rigetti_pyo3::num_complex::Complex32 as NumComplex32;
use rigetti_pyo3::{py_wrap_data_struct, py_wrap_union_enum, PyTryFrom, ToPython};

py_wrap_data_struct! {
    PyReadoutValues(ReadoutValues) as "ReadoutValues" {
        values: Option<Values> => Option<PyReadoutValuesValues>
    }
}

py_wrap_union_enum! {
    PyReadoutValuesValues(Values) as "ReadoutValuesValues" {
        integer_values: IntegerValues => PyIntegerReadoutValues,
        complex_values: ComplexValues => PyComplexReadoutValues
    }
}

py_wrap_data_struct! {
    PyIntegerReadoutValues(IntegerReadoutValues) as "IntegerReadoutValues" {
        values: Vec<i32> => Vec<Py<PyInt>>
    }
}

#[repr(transparent)]
#[derive(Clone)]
struct Complex64Wrapper(Complex64);

impl From<NumComplex32> for Complex64Wrapper {
    fn from(value: NumComplex32) -> Self {
        Self(Complex64 {
            real: Some(value.re),
            imaginary: Some(value.im),
        })
    }
}

impl From<Complex64Wrapper> for NumComplex32 {
    fn from(value: Complex64Wrapper) -> Self {
        Self {
            re: value.0.real.unwrap_or_default(),
            im: value.0.imaginary.unwrap_or_default(),
        }
    }
}

impl ToPyObject for Complex64Wrapper {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        NumComplex32::from(self.clone()).to_object(py)
    }
}

impl ToPython<Py<PyComplex>> for Complex64Wrapper {
    fn to_python(&self, py: Python) -> PyResult<Py<PyComplex>> {
        NumComplex32::from(self.clone()).to_python(py)
    }
}

impl PyTryFrom<PyComplex> for Complex64Wrapper {
    fn py_try_from(py: Python, item: &PyComplex) -> PyResult<Self> {
        let complex = NumComplex32::py_try_from(py, item)?;
        Ok(Self::from(complex))
    }
}

impl PyTryFrom<Py<PyComplex>> for Complex64Wrapper {
    fn py_try_from(py: Python, item: &Py<PyComplex>) -> PyResult<Self> {
        let complex = NumComplex32::py_try_from(py, item)?;
        Ok(Self::from(complex))
    }
}

impl PyTryFrom<Complex64Wrapper> for Complex64 {
    fn py_try_from(_py: Python, item: &Complex64Wrapper) -> PyResult<Self> {
        Ok(item.0.clone())
    }
}

impl ToPython<Complex64Wrapper> for Complex64 {
    fn to_python(&self, _py: Python) -> PyResult<Complex64Wrapper> {
        Ok(Complex64Wrapper(self.clone()))
    }
}

//impl ToPython<Complex64> for Complex64Wrapper {}

impl PyTryFrom<Complex64> for Complex64Wrapper {
    fn py_try_from(_py: Python, item: &Complex64) -> PyResult<Self> {
        Ok(Self(item.clone()))
    }
}

py_wrap_data_struct! {
    PyComplexReadoutValues(Complex64ReadoutValues) as "ComplexReadoutValues" {
        values: Vec<Complex64> => Vec<Complex64Wrapper> => Vec<Py<PyComplex>>
    }
}
