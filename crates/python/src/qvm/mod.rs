use qcs::{qvm::QvmResultData, RegisterData};
use rigetti_pyo3::{
    create_init_submodule, py_wrap_error, py_wrap_type,
    pyo3::{exceptions::PyRuntimeError, prelude::*, Python},
    wrap_error, PyTryFrom, PyWrapper, ToPython, ToPythonError,
};
use std::collections::HashMap;

use crate::{
    py_sync::py_function_sync_async, qpu::client::PyQcsClient, register_data::PyRegisterData,
};

mod api;

use api::PyAddressRequest;

wrap_error!(RustQvmError(qcs::qvm::Error));
py_wrap_error!(api, RustQvmError, QVMError, PyRuntimeError);

py_wrap_type! {
    PyQvmResultData(QvmResultData) as "QVMResultData"
}

create_init_submodule! {
    classes: [PyQvmResultData],
    errors: [QVMError],
    funcs: [py_run, py_run_async],
    submodules: [
        "api": api::init_submodule
    ],
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

py_function_sync_async! {
    #[pyfunction(client = "None")]
    async fn run(
        quil: String,
        shots: u16,
        addresses: HashMap<String, PyAddressRequest>,
        params: HashMap<String, Vec<f64>>,
        client: Option<PyQcsClient>,
    ) -> PyResult<PyQvmResultData> {
        let client = PyQcsClient::get_or_create_client(client).await?;
        let config = client.get_config();
        let params = params.into_iter().map(|(key, value)| (key.into_boxed_str(), value)).collect();
        let addresses = addresses.into_iter().map(|(address, request)| (address, request.as_inner().clone())).collect();
        Ok(PyQvmResultData(qcs::qvm::run(&quil, shots, addresses, &params, &config)
            .await
            .map_err(RustQvmError::from)
            .map_err(RustQvmError::to_py_err)?))
    }
}
