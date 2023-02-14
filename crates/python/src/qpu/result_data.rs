use std::collections::HashMap;

use pyo3::{
    pymethods,
    types::{PyComplex, PyDict, PyFloat, PyInt},
    Py, PyResult, Python,
};
use qcs::qpu::result_data::{QpuResultData, ReadoutValues};
use rigetti_pyo3::{create_init_submodule, py_wrap_data_struct, py_wrap_union_enum, PyTryFrom};

py_wrap_union_enum! {
    PyReadoutValues(ReadoutValues) as "ReadoutValues" {
        integer: Integer => Vec<Py<PyInt>>,
        real: Real => Vec<Py<PyFloat>>,
        complex: Complex => Vec<Py<PyComplex>>
    }
}

py_wrap_data_struct! {
    PyQpuResultData(QpuResultData) as "QPUResultData" {
        mappings: HashMap<String, String> => Py<PyDict>,
        readout_values: HashMap<String, ReadoutValues> => HashMap<String, PyReadoutValues> => Py<PyDict>
    }
}

#[pymethods]
impl PyQpuResultData {
    #[new]
    fn __new__(
        py: Python<'_>,
        mappings: HashMap<String, String>,
        readout_values: HashMap<String, PyReadoutValues>,
    ) -> PyResult<Self> {
        Ok(Self(QpuResultData {
            mappings: HashMap::<String, String>::py_try_from(py, &mappings)?,
            readout_values: HashMap::<String, ReadoutValues>::py_try_from(py, &readout_values)?,
        }))
    }
}

create_init_submodule! {
    classes: [PyReadoutValues, PyQpuResultData],
}
