use pyo3::types::PyDict;
use qcs::{qvm::QvmResultData, RegisterData};
use rigetti_pyo3::{py_wrap_data_struct, pyo3::Py};
use std::collections::HashMap;

use crate::register_data::PyRegisterData;

py_wrap_data_struct! {
    PyQvmResultData(QvmResultData) as "QVMResultData" {
        memory: HashMap<String, RegisterData> => HashMap<String, PyRegisterData> => Py<PyDict>
    }
}
