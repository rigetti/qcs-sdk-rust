use std::collections::HashMap;

use pyo3::{
    pymethods,
    types::{PyComplex, PyDict, PyFloat, PyInt},
    Py, PyAny, PyResult, Python, ToPyObject,
};
use qcs::qpu::{QpuResultData, ReadoutValues};
use rigetti_pyo3::{py_wrap_type, py_wrap_union_enum, PyTryFrom, PyWrapper, ToPython};

py_wrap_union_enum! {
    PyReadoutValues(ReadoutValues) as "ReadoutValues" {
        integer: Integer => Vec<Py<PyInt>>,
        real: Real => Vec<Py<PyFloat>>,
        complex: Complex => Vec<Py<PyComplex>>
    }
}

py_wrap_type! {
    PyQpuResultData(QpuResultData) as "QPUResultData"
}

#[pymethods]
impl PyQpuResultData {
    #[new]
    fn __new__(
        py: Python<'_>,
        mappings: HashMap<String, String>,
        readout_values: HashMap<String, PyReadoutValues>,
    ) -> PyResult<Self> {
        Ok(Self(QpuResultData::from_mappings_and_values(
            mappings,
            HashMap::<String, ReadoutValues>::py_try_from(py, &readout_values)?,
        )))
    }

    #[getter]
    fn mappings(&self, py: Python<'_>) -> PyResult<HashMap<String, String>> {
        self.as_inner().mappings().to_python(py)
    }

    #[getter]
    fn readout_values(&self, py: Python<'_>) -> PyResult<HashMap<String, PyReadoutValues>> {
        self.as_inner().readout_values().to_python(py)
    }

    fn asdict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("mappings", self.mappings(py)?)?;
        dict.set_item(
            "readout_values",
            self.as_inner()
                .readout_values()
                .iter()
                .map(|(register, values)| {
                    (
                        register.to_string(),
                        match values {
                            ReadoutValues::Integer(values) => values.to_object(py),
                            ReadoutValues::Real(values) => values.to_object(py),
                            ReadoutValues::Complex(values) => values.to_object(py),
                        },
                    )
                })
                .collect::<HashMap<String, Py<PyAny>>>(),
        )?;
        Ok(dict.into())
    }
}
