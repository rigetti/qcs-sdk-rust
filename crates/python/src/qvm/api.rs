use std::collections::HashMap;

use super::RustQvmError;
use crate::{
    py_sync::py_function_sync_async, qpu::client::PyQcsClient, register_data::PyRegisterData,
};

use pyo3::{
    pymethods,
    types::{PyBool, PyInt, PyString},
    Py, Python,
};
use qcs::{
    qvm::api::{AddressRequest, MultishotRequest, MultishotResponse},
    RegisterData,
};
use rigetti_pyo3::{
    create_init_submodule, py_wrap_data_struct, py_wrap_union_enum,
    pyo3::{pyfunction, PyResult},
    PyTryFrom, PyWrapper, ToPythonError,
};

create_init_submodule! {
    funcs: [
        py_get_version_info,
        py_get_version_info_async,
        py_run,
        py_run_async
    ],
}

py_function_sync_async! {
    #[pyfunction(client = "None")]
    async fn get_version_info(client: Option<PyQcsClient>) -> PyResult<String> {
        let client = PyQcsClient::get_or_create_client(client).await?;
        let config = client.get_config();
        qcs::qvm::api::get_version_info(&config)
            .await
            .map_err(RustQvmError::from)
            .map_err(RustQvmError::to_py_err)
    }
}

py_wrap_union_enum! {
    PyAddressRequest(AddressRequest) as "AddressRequest" {
        all: All => Py<PyBool>,
        indices: Indices => Vec<Py<PyInt>>
    }
}

py_wrap_data_struct! {
    #[derive(Debug, PartialEq, Eq)]
    PyMultishotRequest(MultishotRequest) as "MultishotRequest" {
        quil_instructions: String => Py<PyString>,
        addresses: HashMap<String, AddressRequest> => HashMap<String, PyAddressRequest>,
        trials: u16 => Py<PyInt>
    }
}

#[pymethods]
impl PyMultishotRequest {
    #[new]
    pub fn new(
        py: Python<'_>,
        program: &str,
        shots: u16,
        addresses: HashMap<String, PyAddressRequest>,
    ) -> PyResult<Self> {
        Ok(Self(MultishotRequest::new(
            program,
            shots,
            HashMap::<String, AddressRequest>::py_try_from(py, &addresses)?,
        )))
    }
}

py_wrap_data_struct! {
    #[derive(Debug, PartialEq)]
    PyMultishotResponse(MultishotResponse) as "MultishotResponse" {
        registers: HashMap<String, RegisterData> => HashMap<String, PyRegisterData>
    }
}

py_function_sync_async! {
    #[pyfunction(client = "None")]
    async fn run(
        request: PyMultishotRequest,
        client: Option<PyQcsClient>,
    ) -> PyResult<PyMultishotResponse> {
        let client = PyQcsClient::get_or_create_client(client).await?;
        let config = client.get_config();
        qcs::qvm::api::run(request.as_inner(), &config)
            .await
            .map_err(RustQvmError::from)
            .map_err(RustQvmError::to_py_err)
            .map(|response| PyMultishotResponse(response))
    }
}
