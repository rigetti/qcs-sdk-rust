use qcs::{qvm::QvmResultData, RegisterData};
use rigetti_pyo3::{
    create_init_submodule, py_wrap_error, py_wrap_type,
    pyo3::{exceptions::PyRuntimeError, prelude::*, Python},
    wrap_error, PyTryFrom, PyWrapper, ToPython,
};
use std::collections::HashMap;

use crate::register_data::PyRegisterData;

mod api;

wrap_error!(RustQvmError(qcs::qvm::Error));
py_wrap_error!(api, RustQvmError, QVMError, PyRuntimeError);

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
    errors: [QVMError],
    submodules: [
        "api": api::init_submodule
    ],
}
