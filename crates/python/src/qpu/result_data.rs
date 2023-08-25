use std::collections::HashMap;

use pyo3::{
    pyclass, pymethods,
    types::{PyComplex, PyFloat, PyInt, PyList},
    IntoPy, Py, PyResult, Python,
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

    pub(crate) fn to_raw_readout_data(&self, py: Python<'_>) -> RawQpuReadoutData {
        RawQpuReadoutData {
            mappings: self.as_inner().mappings().clone(),
            readout_values: self
                .as_inner()
                .readout_values()
                .iter()
                .map(|(register, values)| {
                    (
                        register.to_string(),
                        match values {
                            ReadoutValues::Integer(values) => PyList::new(py, values).into_py(py),
                            ReadoutValues::Real(values) => PyList::new(py, values).into_py(py),
                            ReadoutValues::Complex(values) => PyList::new(py, values).into_py(py),
                        },
                    )
                })
                .collect::<HashMap<String, Py<PyList>>>(),
        }
    }
}

/// A wrapper type for data returned by the QPU in a more flat structure than
/// [`PyQpuResultData`] offers. This makes it more convenient to work with
/// the data if you don't care what type of number the readout values for
/// each register contains.
#[derive(Debug)]
#[pyclass(name = "RawQPUReadoutData")]
pub struct RawQpuReadoutData {
    #[pyo3(get)]
    mappings: HashMap<String, String>,
    #[pyo3(get)]
    readout_values: HashMap<String, Py<PyList>>,
}

impl RawQpuReadoutData {
    fn __repr__(&self) -> String {
        format!("{self:?}")
    }
}
