use numpy::PyArray;
use pyo3::{
    prelude::*,
    types::{PyComplex, PyInt, PyList},
};
use qcs_api_client_grpc::models::controller::{
    readout_values::Values, Complex64, Complex64ReadoutValues, IntegerReadoutValues, ReadoutValues,
};
use rigetti_pyo3::{num_complex::Complex32 as NumComplex32, py_wrap_struct, PyWrapper};
use rigetti_pyo3::{py_wrap_data_struct, py_wrap_union_enum, PyTryFrom, ToPython};

py_wrap_data_struct! {
    PyReadoutValues(ReadoutValues) as "ReadoutValues" {
        values: Option<Values> => Option<PyReadoutValuesValues>
    }
}

#[pymethods]
impl PyReadoutValues {
    pub fn as_ndarray(&self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        match &self.as_inner().values {
            None => Ok(None),
            Some(Values::IntegerValues(ints)) => {
                Ok(Some(PyArray::from_slice(py, &ints.values).to_object(py)))
            }
            Some(Values::ComplexValues(complex)) => Ok(Some(
                PyArray::from_iter(py, {
                    complex
                        .values
                        .iter()
                        .map(|value| NumComplex32::new(value.real, value.imaginary))
                })
                .to_object(py),
            )),
        }
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

py_wrap_struct! {
    PyComplexReadoutValues(Complex64ReadoutValues) as "ComplexReadoutValues" {
        py -> rs {
            list: Py<PyList> => Complex64ReadoutValues {
                let list = <Vec<Py<PyComplex>>>::py_try_from(py, &list)?;
                let values = list.into_iter().map(|complex| {
                    let complex = NumComplex32::py_try_from(py, &complex)?;
                    Ok::<_, PyErr>(Complex64 {
                        real: complex.re,
                        imaginary: complex.im,
                    })
                }).collect::<Result<_, _>>()?;

                Ok::<_, PyErr>(Complex64ReadoutValues { values })
            }
        },
        rs -> py {
            values: Complex64ReadoutValues => Py<PyList> {
                let list = values.values.into_iter().map(|complex| {
                    NumComplex32 {
                        re: complex.real,
                        im: complex.imaginary,
                    }
                }).collect::<Vec<_>>();

                list.to_python(py)
            }
        }
    }
}
