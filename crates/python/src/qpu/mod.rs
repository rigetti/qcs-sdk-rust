use std::time::Duration;

use pyo3::{exceptions::PyRuntimeError, pyfunction, PyResult};
use rigetti_pyo3::{create_init_submodule, py_wrap_error, wrap_error, ToPythonError};

pub use result_data::{PyQpuResultData, PyReadoutValues, RawQpuReadoutData};

pub mod api;
pub mod isa;
mod result_data;
pub mod rewrite_arithmetic;
pub mod translation;

use crate::client::PyQcsClient;
use crate::py_sync::py_function_sync_async;

create_init_submodule! {
    classes: [
        PyQpuResultData,
        RawQpuReadoutData,
        PyReadoutValues
    ],
    errors: [
        ListQuantumProcessorsError
    ],
    funcs: [
        py_list_quantum_processors,
        py_list_quantum_processors_async
    ],
    submodules: [
        "api": api::init_submodule,
        "isa": isa::init_submodule,
        "rewrite_arithmetic": rewrite_arithmetic::init_submodule,
        "translation": translation::init_submodule
    ],
}

wrap_error!(RustListQuantumProcessorsError(
    qcs::qpu::ListQuantumProcessorsError
));
py_wrap_error!(
    api,
    RustListQuantumProcessorsError,
    ListQuantumProcessorsError,
    PyRuntimeError
);

py_function_sync_async! {
    #[pyfunction]
    #[pyo3(signature = (client = None, timeout = None))]
    async fn list_quantum_processors(
        client: Option<PyQcsClient>,
        timeout: Option<f64>,
    ) -> PyResult<Vec<String>> {
        let client = PyQcsClient::get_or_create_client(client).await;
        let timeout = timeout.map(Duration::from_secs_f64);
        qcs::qpu::list_quantum_processors(&client, timeout)
            .await
            .map_err(RustListQuantumProcessorsError::from)
            .map_err(RustListQuantumProcessorsError::to_py_err)
    }
}
