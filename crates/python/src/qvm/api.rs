use super::RustQvmError;
use crate::{py_sync::py_function_sync_async, qpu::client::PyQcsClient};

use rigetti_pyo3::{
    create_init_submodule,
    pyo3::{pyfunction, PyResult},
    ToPythonError,
};

create_init_submodule! {
    funcs: [
        py_get_version_info,
        py_get_version_info_async
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
