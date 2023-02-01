use qcs::{qvm::QvmResultData, RegisterData};
use rigetti_pyo3::{
    create_init_submodule, py_wrap_data_struct,
    pyo3::{prelude::*, types::PyDict, Py, Python},
    PyTryFrom,
};
use std::collections::HashMap;

use crate::register_data::PyRegisterData;

py_wrap_data_struct! {
    PyQvmResultData(QvmResultData) as "QVMResultData" {
        memory: HashMap<String, RegisterData> => HashMap<String, PyRegisterData> => Py<PyDict>
    }
}

#[pymethods]
impl PyQvmResultData {
    #[staticmethod]
    fn from_memory_map(py: Python<'_>, memory: HashMap<String, PyRegisterData>) -> PyResult<Self> {
        Ok(Self(QvmResultData {
            memory: HashMap::<String, RegisterData>::py_try_from(py, &memory)?,
        }))
    }
}

create_init_submodule! {
    classes: [PyQvmResultData],
}
