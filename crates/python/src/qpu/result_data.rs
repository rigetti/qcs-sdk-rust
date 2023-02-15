use std::collections::HashMap;

use pyo3::{
    pymethods,
    types::{PyComplex, PyFloat, PyInt},
    Py, PyResult, Python,
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
}
