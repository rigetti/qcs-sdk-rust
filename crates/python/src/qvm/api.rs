use std::collections::HashMap;

use crate::qvm::PyQvmResultData;
use crate::{py_sync::py_function_sync_async, qpu::client::PyQcsClient};

use rigetti_pyo3::{
    create_init_submodule, py_wrap_error,
    pyo3::{exceptions::PyRuntimeError, pyfunction, PyResult},
    wrap_error, ToPythonError,
};

create_init_submodule! {
    errors: [
        QvmError
    ],
    funcs: [
        py_run,
        py_run_async
    ],
}

wrap_error!(RustQvmError(qcs::qvm::Error));
py_wrap_error!(api, RustQvmError, QvmError, PyRuntimeError);

py_function_sync_async! {
    #[pyfunction(config = "None")]
    async fn run(
        quil: String,
        shots: u16,
        readouts: String,
        params: HashMap<String, Vec<f64>>,
        client: Option<PyQcsClient>,
    ) -> PyResult<PyQvmResultData> {
        let client = PyQcsClient::get_or_create_client(client).await?;
        let config = client.get_config();
        let params = params.into_iter().map(|(key, value)| (key.into_boxed_str(), value)).collect();
        Ok(PyQvmResultData(qcs::qvm::api::run(&quil, shots, readouts, &params, &config)
            .await
            .map_err(RustQvmError::from)
            .map_err(RustQvmError::to_py_err)?))
    }
}
