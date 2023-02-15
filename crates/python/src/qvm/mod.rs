use qcs::{qvm::QvmResultData, RegisterData};
use rigetti_pyo3::{
    create_init_submodule, py_wrap_type,
    pyo3::{prelude::*, Python},
    PyTryFrom, PyWrapper, ToPython,
};
use std::collections::HashMap;

use crate::register_data::PyRegisterData;

py_wrap_type! {
    PyQvmResultData(QvmResultData) as "QVMResultData"
}

#[pymethods]
impl PyQvmResultData {
    #[staticmethod]
    fn from_memory_map(py: Python<'_>, memory: HashMap<String, PyRegisterData>) -> PyResult<Self> {
        Ok(Self(QvmResultData::from_memory_map(HashMap::<
            String,
            RegisterData,
        >::py_try_from(
            py, &memory
        )?)))
    }

    #[getter]
    fn memory(&self, py: Python<'_>) -> PyResult<HashMap<String, PyRegisterData>> {
        self.as_inner().memory().to_python(py)
    }
}

create_init_submodule! {
    classes: [PyQvmResultData],
}
