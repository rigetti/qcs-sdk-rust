//! Running programs on a QPU.
use std::collections::HashMap;

use pyo3::{exceptions::PyRuntimeError, pyfunction, PyResult};
use qcs::qpu::client::GrpcClientError;
use rigetti_pyo3::{create_init_submodule, py_wrap_error, ToPythonError};

use crate::py_sync::py_function_sync_async;

use super::client::PyQcsClient;

create_init_submodule! {
    errors: [
        PySubmitError
    ],
    funcs: [
        py_submit,
        py_submit_async
    ],
}

/// Errors that may occur when submitting a program for execution
#[derive(Debug, thiserror::Error)]
pub enum SubmitError {
    /// Failed a gRPC API call
    #[error("Failed a gRPC call: {0}")]
    GrpcError(#[from] GrpcClientError),

    /// Job could not be deserialized
    #[error("Failed to deserialize job: {0}")]
    DeserializeError(#[from] serde_json::Error),
}

py_wrap_error!(runner, SubmitError, PySubmitError, PyRuntimeError);

py_function_sync_async! {
    /// Submits an executable `program` to be run on the specified QPU
    ///
    /// # Errors
    ///
    /// May return an error if
    /// * an engagement is not available
    /// * an RPCQ client cannot be built
    /// * the program cannot be submitted
    #[allow(clippy::implicit_hasher)]
    #[pyfunction(client = "None")]
    async fn submit(
        program: String,
        patch_values: HashMap<String, Vec<f64>>,
        quantum_processor_id: String,
        client: Option<PyQcsClient>,
    ) -> PyResult<String> {
        let client = PyQcsClient::get_or_create_client(client).await?;

        // Is there a better way to map these patch_values keys? This
        // negates the whole purpose of [`submit`] using `Box<str>`,
        // instead of `String` directly, which normally would decrease
        // copies _and_ require less space, since str can't be extended.
        let patch_values = patch_values
            .into_iter()
            .map(|(k, v)| (k.into_boxed_str(), v))
            .collect();

        let job = serde_json::from_str(&program)
            .map_err(SubmitError::from)
            .map_err(SubmitError::to_py_err)?;

        let job_id = qcs::qpu::runner::submit(&quantum_processor_id, job, &patch_values, &client).await
            .map_err(SubmitError::from)
            .map_err(SubmitError::to_py_err)?;

        Ok(job_id.to_string())
    }
}
