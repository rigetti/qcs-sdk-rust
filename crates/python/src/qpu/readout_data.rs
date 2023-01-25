use std::collections::HashMap;

use pyo3::{
    types::{PyComplex, PyDict, PyFloat, PyInt},
    Py,
};
use qcs::qpu::readout_data::{QpuReadout, ReadoutValues};
use rigetti_pyo3::{py_wrap_data_struct, py_wrap_union_enum};

py_wrap_union_enum! {
    PyReadoutValues(ReadoutValues) as "ReadoutValues" {
        integer: Integer => Vec<Py<PyInt>>,
        real: Real => Vec<Py<PyFloat>>,
        complex: Complex => Vec<Py<PyComplex>>
    }
}

py_wrap_data_struct! {
    PyQpuReadout(QpuReadout) as "QPUReadout" {
        mappings: HashMap<String, String> => Py<PyDict>,
        readout_values: HashMap<String, ReadoutValues> => HashMap<String, PyReadoutValues> => Py<PyDict>
    }
}
